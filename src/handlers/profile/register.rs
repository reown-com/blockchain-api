use {
    super::{
        super::HANDLER_TASK_METRICS,
        utils::{
            check_attributes,
            is_name_format_correct,
            is_name_in_allowed_zones,
            is_timestamp_within_interval,
        },
        Eip155SupportedChains,
        RegisterPayload,
        RegisterRequest,
        ALLOWED_ZONES,
        UNIXTIMESTAMP_SYNC_THRESHOLD,
    },
    crate::{
        database::{
            helpers::{get_name_and_addresses_by_name, insert_name},
            types::{Address, ENSIP11AddressesMap, SupportedNamespaces},
        },
        error::RpcError,
        state::AppState,
        utils::crypto::verify_message_signature,
    },
    axum::{
        extract::State,
        response::{IntoResponse, Response},
        Json,
    },
    hyper::StatusCode,
    num_enum::TryFromPrimitive,
    sqlx::Error as SqlxError,
    std::{collections::HashMap, str::FromStr, sync::Arc},
    tracing::log::{error, info},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    Json(register_request): Json<RegisterRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, register_request)
        .with_metrics(HANDLER_TASK_METRICS.with_name("profile_register"))
        .await
}

#[tracing::instrument(skip(state))]
pub async fn handler_internal(
    state: State<Arc<AppState>>,
    register_request: RegisterRequest,
) -> Result<Response, RpcError> {
    let raw_payload = &register_request.message;
    let payload = match serde_json::from_str::<RegisterPayload>(raw_payload) {
        Ok(payload) => payload,
        Err(e) => {
            info!("Failed to deserialize register payload: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "").into_response());
        }
    };

    // Check if the name is in the correct format
    if !is_name_format_correct(&payload.name) {
        info!("Invalid name format: {}", payload.name);
        return Ok((StatusCode::BAD_REQUEST, "Invalid name format").into_response());
    }

    // Check is name in the allowed zones
    if !is_name_in_allowed_zones(&payload.name, &ALLOWED_ZONES) {
        info!("Name is not in the allowed zones");
        return Ok((StatusCode::BAD_REQUEST, "Name is not in the allowed zones").into_response());
    }

    // Check for the supported ENSIP-11 coin type
    if Eip155SupportedChains::try_from_primitive(register_request.coin_type).is_err() {
        info!("Unsupported coin type {}", register_request.coin_type);
        return Ok((
            StatusCode::BAD_REQUEST,
            "Unsupported coin type for name registration",
        )
            .into_response());
    }

    // Check is name already registered
    if get_name_and_addresses_by_name(payload.name.clone(), &state.postgres.clone())
        .await
        .is_ok()
    {
        info!(
            "Registration request for already registered name {}",
            payload.name.clone()
        );
        return Ok((StatusCode::BAD_REQUEST, "Name is already registered").into_response());
    };

    // Check the timestamp is within the sync threshold interval
    if !is_timestamp_within_interval(payload.timestamp, UNIXTIMESTAMP_SYNC_THRESHOLD) {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Timestamp is too old or in the future",
        )
            .into_response());
    }

    let owner = match ethers::types::H160::from_str(&register_request.address) {
        Ok(owner) => owner,
        Err(e) => {
            info!("Failed to parse H160 address: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "Invalid H160 address format").into_response());
        }
    };

    // Check for supported attributes
    if let Some(attributes) = payload.attributes.clone() {
        if !check_attributes(
            &attributes,
            &super::SUPPORTED_ATTRIBUTES,
            super::ATTRIBUTES_VALUE_MAX_LENGTH,
        ) {
            return Ok((
                StatusCode::BAD_REQUEST,
                "Unsupported attribute in
        payload",
            )
                .into_response());
        }
    }

    // Check the signature
    let sinature_check =
        match verify_message_signature(raw_payload, &register_request.signature, &owner) {
            Ok(sinature_check) => sinature_check,
            Err(e) => {
                info!("Invalid signature: {}", e);
                return Ok((
                    StatusCode::UNAUTHORIZED,
                    "Invalid signature or message format",
                )
                    .into_response());
            }
        };
    if !sinature_check {
        return Ok((StatusCode::UNAUTHORIZED, "Signature verification error").into_response());
    }

    // Register (insert) a new domain with address
    let addresses: ENSIP11AddressesMap = HashMap::from([(register_request.coin_type, Address {
        address: register_request.address,
        created_at: None,
    })]);

    let insert_result = insert_name(
        payload.name.clone(),
        payload.attributes.unwrap_or(HashMap::new()),
        SupportedNamespaces::Eip155,
        addresses,
        &state.postgres,
    )
    .await;
    if let Err(e) = insert_result {
        error!("Failed to insert new name: {}", e);
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
    }

    // Return the registered name and addresses
    match get_name_and_addresses_by_name(payload.name, &state.postgres.clone()).await {
        Ok(response) => Ok(Json(response).into_response()),
        Err(e) => match e {
            SqlxError::RowNotFound => {
                error!("New registered name is not found in the database: {}", e);
                Ok((StatusCode::INTERNAL_SERVER_ERROR, "Name is not registered").into_response())
            }
            _ => {
                error!("Error on lookup new registered name: {}", e);
                Ok((StatusCode::INTERNAL_SERVER_ERROR, "Name is not registered").into_response())
            }
        },
    }
}
