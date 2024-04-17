use {
    super::{
        super::HANDLER_TASK_METRICS,
        utils::is_timestamp_within_interval,
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
        utils::crypto::{constant_time_eq, is_coin_type_supported, verify_message_signature},
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::types::H160,
    hyper::StatusCode,
    sqlx::Error as SqlxError,
    std::{str::FromStr, sync::Arc},
    tracing::log::error,
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
        Err(e) => return Err(RpcError::SerdeJson(e)),
    };

    // Check for the supported ENSIP-11 coin type
    if !is_coin_type_supported(request_payload.coin_type) {
        return Err(RpcError::UnsupportedCoinType(request_payload.coin_type));
    }

    // Check is name registered
    let name_addresses =
        match get_name_and_addresses_by_name(name.clone(), &state.postgres.clone()).await {
            Ok(result) => result,
            Err(e) => match e {
                SqlxError::RowNotFound => return Err(RpcError::NameNotRegistered(name)),
                _ => {
                    error!("Failed to lookup name in the database: {}", e);
                    return Ok((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Name lookup database error",
                    )
                        .into_response());
                }
            },
        };

    // Check the timestamp is within the sync threshold interval
    if !is_timestamp_within_interval(payload.timestamp, UNIXTIMESTAMP_SYNC_THRESHOLD) {
        return Err(RpcError::ExpiredTimestamp(payload.timestamp));
    }

    let payload_owner = match H160::from_str(&request_payload.address) {
        Ok(owner) => owner,
        Err(_) => return Err(RpcError::InvalidAddress),
    };

    // Check the signature
    let sinature_check =
        match verify_message_signature(raw_payload, &request_payload.signature, &payload_owner) {
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

    // Check for the name address ownership and address from the signed payload
    let mut address_is_authorized = false;
    for (coint_type, address) in name_addresses.addresses.iter() {
        if coint_type == &request_payload.coin_type {
            let name_owner = match ethers::types::H160::from_str(&address.address) {
                Ok(owner) => owner,
                Err(_) => return Err(RpcError::InvalidAddress),
            };
            if !constant_time_eq(payload_owner, name_owner) {
                return Err(RpcError::NameOwnerValidationError);
            } else {
                address_is_authorized = true;
            }
        }
    }
    if !address_is_authorized {
        return Err(RpcError::NameOwnerValidationError);
    }

    // Check for supported chain id and address format
    if !is_coin_type_supported(payload.coin_type) {
        return Err(RpcError::UnsupportedCoinType(payload.coin_type));
    }

    // Check the new address format
    if H160::from_str(&payload.address).is_err() {
        return Err(RpcError::InvalidAddress);
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
