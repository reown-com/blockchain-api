use {
    super::{
        super::HANDLER_TASK_METRICS,
        utils::{check_attributes, is_timestamp_within_interval},
        Eip155SupportedChains,
        RegisterRequest,
        UpdateAttributesPayload,
        UNIXTIMESTAMP_SYNC_THRESHOLD,
    },
    crate::{
        database::helpers::{get_name_and_addresses_by_name, update_name_attributes},
        error::RpcError,
        state::AppState,
        utils::crypto::{constant_time_eq, verify_message_signature},
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::StatusCode,
    num_enum::TryFromPrimitive,
    std::{str::FromStr, sync::Arc},
    tracing::log::{error, info},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    name: Path<String>,
    Json(request_payload): Json<RegisterRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, name, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("profile_attributes_update"))
        .await
}

#[tracing::instrument(skip(state))]
pub async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(name): Path<String>,
    request_payload: RegisterRequest,
) -> Result<Response, RpcError> {
    let raw_payload = &request_payload.message;
    let payload = match serde_json::from_str::<UpdateAttributesPayload>(raw_payload) {
        Ok(payload) => payload,
        Err(e) => {
            info!("Failed to deserialize update attributes payload: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "").into_response());
        }
    };

    // Check for the supported ENSIP-11 coin type
    if Eip155SupportedChains::try_from_primitive(request_payload.coin_type).is_err() {
        info!("Unsupported coin type {}", request_payload.coin_type);
        return Ok((
            StatusCode::BAD_REQUEST,
            "Unsupported coin type for name attributes update",
        )
            .into_response());
    }

    // Check is name registered
    let name_addresses =
        match get_name_and_addresses_by_name(name.clone(), &state.postgres.clone()).await {
            Ok(result) => result,
            Err(_) => {
                info!(
                    "Update attributes request for not registered name {}",
                    name.clone()
                );
                return Ok((StatusCode::BAD_REQUEST, "Name is not registered").into_response());
            }
        };

    // Check the timestamp is within the sync threshold interval
    if !is_timestamp_within_interval(payload.timestamp, UNIXTIMESTAMP_SYNC_THRESHOLD) {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Timestamp is too old or in the future",
        )
            .into_response());
    }

    let payload_owner = match ethers::types::H160::from_str(&request_payload.address) {
        Ok(owner) => owner,
        Err(e) => {
            info!("Failed to parse H160 address: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "Invalid H160 address format").into_response());
        }
    };

    // Check the signature
    let sinature_check =
        match verify_message_signature(raw_payload, &request_payload.signature, &payload_owner) {
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

    // Check for the name address ownership and address from the signed payload
    let name_owner = match name_addresses
        .addresses
        .get(&Eip155SupportedChains::Mainnet.into())
    {
        Some(address_entry) => match ethers::types::H160::from_str(&address_entry.address) {
            Ok(owner) => owner,
            Err(e) => {
                info!("Failed to parse H160 address: {}", e);
                return Ok((StatusCode::BAD_REQUEST, "Invalid H160 address format").into_response());
            }
        },
        None => {
            info!("Address entry not found for key 60");
            return Ok((
                StatusCode::BAD_REQUEST,
                "Address entry not found for key 60",
            )
                .into_response());
        }
    };
    if !constant_time_eq(payload_owner, name_owner) {
        return Ok((
            StatusCode::UNAUTHORIZED,
            "Address is not the owner of the name",
        )
            .into_response());
    }

    // Check for supported attributes
    if !check_attributes(
        &payload.attributes,
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

    match update_name_attributes(name.clone(), payload.attributes, &state.postgres).await {
        Err(e) => {
            error!("Failed to update attributes: {}", e);
            Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response())
        }
        Ok(attributes) => Ok(Json(attributes).into_response()),
    }
}
