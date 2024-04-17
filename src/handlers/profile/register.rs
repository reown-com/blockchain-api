use {
    super::{
        super::HANDLER_TASK_METRICS,
        utils::{
            check_attributes,
            is_name_format_correct,
            is_name_in_allowed_zones,
            is_name_length_correct,
            is_timestamp_within_interval,
        },
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
        utils::crypto::{is_coin_type_supported, verify_message_signature},
    },
    axum::{
        extract::State,
        response::{IntoResponse, Response},
        Json,
    },
    hyper::StatusCode,
    sqlx::Error as SqlxError,
    std::{collections::HashMap, str::FromStr, sync::Arc},
    tracing::log::error,
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
        Err(e) => return Err(RpcError::SerdeJson(e)),
    };

    // Check if the name is in the correct format
    if !is_name_format_correct(&payload.name) {
        return Err(RpcError::InvalidNameFormat(payload.name));
    }

    // Check if the name length is correct
    if !is_name_length_correct(&payload.name) {
        return Err(RpcError::InvalidNameLength(payload.name));
    }

    // Check is name in the allowed zones
    if !is_name_in_allowed_zones(&payload.name, &ALLOWED_ZONES) {
        return Err(RpcError::InvalidNameZone(payload.name));
    }

    // Check for the supported ENSIP-11 coin type
    if !is_coin_type_supported(register_request.coin_type) {
        return Err(RpcError::UnsupportedCoinType(register_request.coin_type));
    }

    // Check is name already registered
    if get_name_and_addresses_by_name(payload.name.clone(), &state.postgres.clone())
        .await
        .is_ok()
    {
        return Err(RpcError::NameAlreadyRegistered(payload.name.clone()));
    };

    // Check the timestamp is within the sync threshold interval
    if !is_timestamp_within_interval(payload.timestamp, UNIXTIMESTAMP_SYNC_THRESHOLD) {
        return Err(RpcError::ExpiredTimestamp(payload.timestamp));
    }

    let owner = match ethers::types::H160::from_str(&register_request.address) {
        Ok(owner) => owner,
        Err(_) => return Err(RpcError::InvalidAddress),
    };

    // Check for supported attributes
    if let Some(attributes) = payload.attributes.clone() {
        if !check_attributes(
            &attributes,
            &super::SUPPORTED_ATTRIBUTES,
            super::ATTRIBUTES_VALUE_MAX_LENGTH,
        ) {
            return Err(RpcError::UnsupportedNameAttribute);
        }
    }

    // Check the signature
    let sinature_check =
        match verify_message_signature(raw_payload, &register_request.signature, &owner) {
            Ok(sinature_check) => sinature_check,
            Err(_) => {
                return Err(RpcError::SignatureValidationError(
                    "Invalid signature".into(),
                ))
            }
        };
    if !sinature_check {
        return Err(RpcError::SignatureValidationError(
            "Signature verification error".into(),
        ));
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
    match get_name_and_addresses_by_name(payload.name.clone(), &state.postgres.clone()).await {
        Ok(response) => Ok(Json(response).into_response()),
        Err(e) => match e {
            SqlxError::RowNotFound => Err(RpcError::NameNotFound(payload.name.clone())),
            _ => {
                // Handle other types of errors
                error!("Failed to lookup name: {}", e);
                return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
            }
        },
    }
}
