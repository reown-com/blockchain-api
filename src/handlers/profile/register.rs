use {
    super::{
        super::HANDLER_TASK_METRICS,
        verify_message_signature,
        RegisterPayload,
        RegisterRequest,
    },
    crate::{
        database::{
            helpers::{get_name_and_addresses_by_name, insert_name},
            types::{Address, SupportedNamespaces},
        },
        error::RpcError,
        state::AppState,
    },
    axum::{
        body::Bytes,
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::StatusCode,
    sqlx::Error as SqlxError,
    std::{collections::HashMap, str::FromStr, sync::Arc},
    tracing::log::{error, info},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    name: Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    handler_internal(state, name, body)
        .with_metrics(HANDLER_TASK_METRICS.with_name("profile_register"))
        .await
}

#[tracing::instrument(skip(state))]
pub async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(name): Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    // Check the request body format
    let register_request = match serde_json::from_slice::<RegisterRequest>(&body) {
        Ok(register_request_payload) => register_request_payload,
        Err(e) => {
            info!("Failed to deserialize register request: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "").into_response());
        }
    };

    let raw_payload = &register_request.message;
    let payload = match serde_json::from_str::<RegisterPayload>(raw_payload) {
        Ok(payload) => payload,
        Err(e) => {
            info!("Failed to deserialize register payload: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "").into_response());
        }
    };

    if payload.name != name {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Name in payload and path are not equal",
        )
            .into_response());
    }

    if payload.address != register_request.address {
        return Ok((
            StatusCode::BAD_REQUEST,
            "Address in payload request and message are not equal",
        )
            .into_response());
    }

    // Check is name already registered
    if get_name_and_addresses_by_name(name.clone(), &state.postgres.clone())
        .await
        .is_ok()
    {
        info!("Registration request for registered name {}", name.clone());
        return Ok((StatusCode::BAD_REQUEST, "Name is already registered").into_response());
    };

    // TODO: Check the timestamp within 5-10 minutes ttl

    let owner = match ethers::types::H160::from_str(&register_request.address) {
        Ok(owner) => owner,
        Err(e) => {
            info!("Failed to parse H160 address: {}", e);
            return Ok((StatusCode::BAD_REQUEST, "Invalid H160 address format").into_response());
        }
    };

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
    let addresses = vec![Address {
        namespace: SupportedNamespaces::Eip155,
        chain_id: None,
        address: register_request.address,
        created_at: None,
    }];
    let insert_result = insert_name(name.clone(), HashMap::new(), addresses, &state.postgres).await;
    if let Err(e) = insert_result {
        error!("Failed to insert new name: {}", e);
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
    }

    // Return the registered name and addresses
    match get_name_and_addresses_by_name(name, &state.postgres.clone()).await {
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
