use super::get_assets::{self, GetAssetsError};
use super::get_calls_status::{self, GetCallsStatusError};
use super::get_exchange_buy_status::{self, GetExchangeBuyStatusError};
use super::get_exchange_url::{self, GetExchangeUrlError};
use super::get_exchanges::{self, GetExchangesError};
use super::pos::{self, BuildPosTxsError, CheckPosTxError, SupportedNetworksError};
use super::prepare_calls::{self, PrepareCallsError};
use super::send_prepared_calls::{self, SendPreparedCallsError};
use crate::error::RpcError;
use crate::json_rpc::{
    ErrorResponse, JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcResult,
};
use crate::utils::simple_request_json::SimpleRequestJson;
use crate::{
    handlers::{wallet::get_calls_status::QueryParams as CallStatusQueryParams, SdkInfoParams},
    state::AppState,
};
use axum::extract::{ConnectInfo, Query};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, Json};
use hyper::{HeaderMap, StatusCode};
use serde::Deserialize;
use std::error::Error as StdError;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tracing::error;
use wc::metrics::{future_metrics, FutureExt};
use yttrium::wallet_service_api;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WalletQueryParams {
    pub project_id: String,
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
    pub source: Option<String>,
}

// TODO support batch requests (and validate unique RPC IDs)
pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<WalletQueryParams>,
    SimpleRequestJson(request_payload): SimpleRequestJson<JsonRpcRequest>,
) -> Response {
    handler_internal(state, connect_info, headers, query, request_payload)
        .with_metrics(future_metrics!("handler_task", "name" => "wallet"))
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
            let body: Response = if e.should_display_source() {
                let data = EnrichedErrorData {
                    variant: deepest_source_message(&e),
                };
                let err = JsonRpcError::<EnrichedErrorData>::new(
                    request.id,
                    ErrorResponse {
                        code: e.to_json_rpc_error_code(),
                        message: e.to_string().into(),
                        data,
                    },
                );
                Json(err).into_response()
            } else {
                let body = JsonRpcResponse::Error(JsonRpcError::new(
                    request.id,
                    ErrorResponse {
                        code: e.to_json_rpc_error_code(),
                        message: e.to_string().into(),
                        data: None,
                    },
                ));
                Json(body).into_response()
            };

            if  e.is_internal() {
                error!("Internal server error handling wallet RPC request: {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            } else {
                (StatusCode::BAD_REQUEST, body).into_response()
            }
        }
    }
}

pub const WALLET_PREPARE_CALLS: &str = "wallet_prepareCalls";
pub const WALLET_SEND_PREPARED_CALLS: &str = "wallet_sendPreparedCalls";
pub const WALLET_GET_CALLS_STATUS: &str = "wallet_getCallsStatus";
pub const PAY_GET_EXCHANGES: &str = "reown_getExchanges";
pub const PAY_GET_EXCHANGE_URL: &str = "reown_getExchangePayUrl";
pub const PAY_GET_EXCHANGE_BUY_STATUS: &str = "reown_getExchangeBuyStatus";
pub const POS_BUILD_TRANSACTIONS: &str = "wc_pos_buildTransactions";
pub const POS_CHECK_TRANSACTION: &str = "wc_pos_checkTransaction";
pub const POS_SUPPORTED_NETWORKS: &str = "wc_pos_supportedNetworks";

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

    #[error("{PAY_GET_EXCHANGES}: {0}")]
    GetExchanges(GetExchangesError),

    #[error("{PAY_GET_EXCHANGE_URL}: {0}")]
    GetUrl(GetExchangeUrlError),

    #[error("{}: {0}", wallet_service_api::WALLET_GET_ASSETS)]
    GetAssets(GetAssetsError),

    #[error("{PAY_GET_EXCHANGE_BUY_STATUS}: {0}")]
    GetExchangeBuyStatus(GetExchangeBuyStatusError),

    #[error("{POS_BUILD_TRANSACTIONS}: {0}")]
    PosBuildTransactions(#[source] BuildPosTxsError),

    #[error("{POS_CHECK_TRANSACTION}: {0}")]
    PosCheckTransaction(#[source] CheckPosTxError),

    #[error("{POS_SUPPORTED_NETWORKS}: {0}")]
    PosSupportedNetworks(#[source] SupportedNetworksError),

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

fn deepest_source_message(err: &(dyn StdError)) -> String {
    let mut last = err.to_string();
    let mut current = err;
    while let Some(src) = current.source() {
        last = src.to_string();
        current = src;
    }
    last
}

#[derive(serde::Serialize)]
struct EnrichedErrorData {
    variant: String,
}

impl Error {
    fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            Error::InvalidProjectId(_) => -1,
            Error::PrepareCalls(_) => -2, // TODO more specific codes
            Error::SendPreparedCalls(_) => -3, // TODO more specific codes
            Error::GetCallsStatus(_) => -4, // TODO more specific codes
            Error::GetAssets(_) => -5,    // TODO more specific codes
            Error::GetExchanges(_) => -6,
            Error::GetUrl(_) => -7,
            Error::GetExchangeBuyStatus(_) => -8,
            Error::PosBuildTransactions(_) => -9,
            Error::PosCheckTransaction(_) => -10,
            Error::PosSupportedNetworks(_) => -11,
            Error::MethodNotFound => -32601,
            Error::InvalidParams(_) => -32602,
            Error::Internal(_) => -32000,
        }
    }

    fn is_internal(&self) -> bool {
        match self {
            Error::InvalidProjectId(_) => false,
            Error::PrepareCalls(e) => e.is_internal(),
            Error::SendPreparedCalls(e) => e.is_internal(),
            Error::GetCallsStatus(e) => e.is_internal(),
            Error::GetAssets(e) => e.is_internal(),
            Error::GetExchanges(e) => e.is_internal(),
            Error::GetUrl(e) => e.is_internal(),
            Error::GetExchangeBuyStatus(e) => e.is_internal(),
            Error::PosBuildTransactions(e) => e.is_internal(),
            Error::PosCheckTransaction(e) => e.is_internal(),
            Error::PosSupportedNetworks(e) => e.is_internal(),
            Error::MethodNotFound => false,
            Error::InvalidParams(_) => false,
            Error::Internal(_) => true,
        }
    }

    fn should_display_source(&self) -> bool {
        matches!(
            self,
            Error::PosBuildTransactions(_) | Error::PosCheckTransaction(_) | Error::PosSupportedNetworks(_)
        )
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
                Query(CallStatusQueryParams {
                    sdk_info: query.sdk_info,
                }),
            )
            .await
            .map_err(Error::GetCallsStatus)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        wallet_service_api::WALLET_GET_ASSETS => serde_json::to_value(
            &get_assets::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
                connect_info,
                headers,
                Query(get_assets::QueryParams {
                    sdk_info: query.sdk_info,
                }),
            )
            .await
            .map_err(Error::GetAssets)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        PAY_GET_EXCHANGES => serde_json::to_value(
            &get_exchanges::handler(
                state,
                project_id,
                connect_info,
                headers,
                Query(get_exchanges::QueryParams {
                    sdk_info: query.sdk_info,
                    source: query.source,
                }),
                Json(serde_json::from_value(params).map_err(Error::InvalidParams)?),
            )
            .await
            .map_err(Error::GetExchanges)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        PAY_GET_EXCHANGE_URL => serde_json::to_value(
            &get_exchange_url::handler(
                state,
                project_id,
                connect_info,
                headers,
                Query(get_exchange_url::QueryParams {
                    sdk_info: query.sdk_info,
                    source: query.source,
                }),
                Json(serde_json::from_value(params).map_err(Error::InvalidParams)?),
            )
            .await
            .map_err(Error::GetUrl)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        PAY_GET_EXCHANGE_BUY_STATUS => serde_json::to_value(
            &get_exchange_buy_status::handler(
                state,
                project_id,
                connect_info,
                headers,
                Query(get_exchange_buy_status::QueryParams {
                    sdk_info: query.sdk_info,
                    source: query.source,
                }),
                Json(serde_json::from_value(params).map_err(Error::InvalidParams)?),
            )
            .await
            .map_err(Error::GetExchangeBuyStatus)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        POS_BUILD_TRANSACTIONS => serde_json::to_value(
            &pos::build_transactions::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
            )
            .await
            .map_err(Error::PosBuildTransactions)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        POS_CHECK_TRANSACTION => serde_json::to_value(
            &pos::check_transaction::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
            )
            .await
            .map_err(Error::PosCheckTransaction)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        POS_SUPPORTED_NETWORKS => serde_json::to_value(
            &pos::supported_networks::handler(state, project_id)
                .await
                .map_err(Error::PosSupportedNetworks)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        _ => Err(Error::MethodNotFound),
    }
}
