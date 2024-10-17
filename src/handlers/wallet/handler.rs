use super::get_calls_status::{self, GetCallsStatusError};
use super::prepare_calls::{self, PrepareCallsError};
use super::send_prepared_calls::{self, SendPreparedCallsError};
use crate::error::RpcError;
use crate::json_rpc::{
    ErrorResponse, JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcResult,
};
use crate::{handlers::HANDLER_TASK_METRICS, state::AppState};
use axum::extract::{ConnectInfo, Query};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, Json};
use hyper::{HeaderMap, StatusCode};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tracing::error;
use wc::future::FutureExt;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WalletQueryParams {
    pub project_id: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<WalletQueryParams>,
    Json(request_payload): Json<JsonRpcRequest>,
) -> Response {
    handler_internal(state, connect_info, headers, query, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("wallet"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<WalletQueryParams>,
    request: JsonRpcRequest,
) -> Response {
    match handle_rpc(
        state,
        connect_info,
        headers,
        query,
        request.method,
        request.params,
    )
    .await
    {
        Ok(result) => Json(JsonRpcResponse::Result(JsonRpcResult::new(
            request.id, result,
        )))
        .into_response(),
        Err(e) => {
            let is_internal = matches!(e, Error::Internal(_));
            if is_internal {
                error!("Internal server error handling wallet RPC request: {e:?}");
            }
            // TODO these special cases shouldn't be necessary, by remapping
            if matches!(
                e,
                Error::SendPreparedCalls(SendPreparedCallsError::InternalError(_))
            ) {
                error!(
                    "Internal server error handling wallet RPC request (sendPreparedCalls): {e:?}"
                );
            }
            if matches!(e, Error::PrepareCalls(PrepareCallsError::InternalError(_))) {
                error!("Internal server error handling wallet RPC request (prepareCalls): {e:?}");
            }
            let json = Json(JsonRpcResponse::Error(JsonRpcError::new(
                request.id,
                ErrorResponse {
                    code: e.to_json_rpc_error_code(),
                    message: e.to_string().into(),
                    data: None,
                },
            )));
            if is_internal {
                (StatusCode::INTERNAL_SERVER_ERROR, json).into_response()
            } else {
                (StatusCode::BAD_REQUEST, json).into_response()
            }
        }
    }
}

pub const WALLET_PREPARE_CALLS: &str = "wallet_prepareCalls";
pub const WALLET_SEND_PREPARED_CALLS: &str = "wallet_sendPreparedCalls";
pub const WALLET_GET_CALLS_STATUS: &str = "wallet_getCallsStatus";

#[derive(Debug, Error)]
enum Error {
    #[error("Invalid project ID: {0}")]
    InvalidProjectId(RpcError),

    #[error("{WALLET_PREPARE_CALLS}: {0}")]
    PrepareCalls(PrepareCallsError),

    #[error("{WALLET_SEND_PREPARED_CALLS}: {0}")]
    SendPreparedCalls(SendPreparedCallsError),

    #[error("{WALLET_GET_CALLS_STATUS}: {0}")]
    GetCallsStatus(GetCallsStatusError),

    #[error("Method not found")]
    MethodNotFound,

    #[error("Invalid params: {0}")]
    InvalidParams(serde_json::Error),

    #[error("Internal error")]
    Internal(InternalError),
}

#[derive(Debug, Error)]
enum InternalError {
    #[error("Serializing response: {0}")]
    SerializeResponse(serde_json::Error),
}

impl Error {
    fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            Error::InvalidProjectId(_) => -1,
            Error::PrepareCalls(_) => -2, // TODO more specific codes
            Error::SendPreparedCalls(_) => -3, // TODO more specific codes
            Error::GetCallsStatus(_) => -4, // TODO more specific codes
            Error::MethodNotFound => -32601,
            Error::InvalidParams(_) => -32602,
            Error::Internal(_) => -32000,
        }
    }
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handle_rpc(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<WalletQueryParams>,
    method: Arc<str>,
    params: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    let project_id = query.project_id;
    state
        .validate_project_access_and_quota(&project_id)
        .await
        // TODO refactor to differentiate between user and server errors
        .map_err(Error::InvalidProjectId)?;

    match method.as_ref() {
        WALLET_PREPARE_CALLS => serde_json::to_value(
            &prepare_calls::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
            )
            .await
            .map_err(Error::PrepareCalls)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        WALLET_SEND_PREPARED_CALLS => serde_json::to_value(
            &send_prepared_calls::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
            )
            .await
            .map_err(Error::SendPreparedCalls)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        WALLET_GET_CALLS_STATUS => serde_json::to_value(
            &get_calls_status::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
                connect_info,
                headers,
            )
            .await
            .map_err(Error::GetCallsStatus)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        _ => Err(Error::MethodNotFound),
    }
}
