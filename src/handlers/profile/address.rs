use {
    super::{
        super::HANDLER_TASK_METRICS,
        utils::is_timestamp_within_interval,
        Eip155SupportedChains,
        RegisterRequest,
        UpdateAddressPayload,
        UNIXTIMESTAMP_SYNC_THRESHOLD,
    },
    crate::{
        database::{
            helpers::{get_name_and_addresses_by_name, insert_or_update_address},
            types::SupportedNamespaces,
        },
        error::RpcError,
        state::AppState,
        utils::crypto::{constant_time_eq, verify_message_signature},
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::types::H160,
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
        .with_metrics(HANDLER_TASK_METRICS.with_name("profile_address_update"))
        .await
}

#[tracing::instrument(skip(state))]
pub async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(name): Path<String>,
    request_payload: RegisterRequest,
) -> Result<Response, RpcError> {
    let raw_payload = &request_payload.message;
    let payload = match serde_json::from_str::<UpdateAddressPayload>(raw_payload) {
        Ok(payload) => payload,
        Err(e) => {
            info!("Failed to deserialize update address payload: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "").into_response());
        }
    };

    // Check for the supported ENSIP-11 coin type
    if Eip155SupportedChains::try_from_primitive(request_payload.coin_type).is_err() {
        info!("Unsupported coin type {}", request_payload.coin_type);
        return Ok((
            StatusCode::BAD_REQUEST,
            "Unsupported coin type for name address update",
        )
            .into_response());
    }

    // Check is name registered
    let name_addresses =
        match get_name_and_addresses_by_name(name.clone(), &state.postgres.clone()).await {
            Ok(result) => result,
            Err(_) => {
                info!(
                    "Update address request for not registered name {}",
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

    let payload_owner = match H160::from_str(&request_payload.address) {
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
    let name_owner = match name_addresses.addresses.get(&60) {
        Some(address_entry) => match H160::from_str(&address_entry.address) {
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

    // Check for supported chain id and address format
    if Eip155SupportedChains::try_from_primitive(payload.coin_type).is_err() {
        info!(
            "Unsupported coin type for name address update {}",
            payload.coin_type
        );
        return Ok((
            StatusCode::BAD_REQUEST,
            "Unsupported coin type for name address update",
        )
            .into_response());
    }

    // Check the new address format
    if let Err(e) = H160::from_str(&payload.address) {
        info!("Failed to parse H160 address for update: {}", e);
        return Ok((StatusCode::BAD_REQUEST, "Invalid H160 address format").into_response());
    }

    match insert_or_update_address(
        name.clone(),
        SupportedNamespaces::Eip155,
        format!("{}", payload.coin_type),
        payload.address,
        &state.postgres.clone(),
    )
    .await
    {
        Ok(response) => Ok(Json(response).into_response()),
        Err(e) => {
            error!("Failed to update address: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update address",
            )
                .into_response())
        }
    }
}
