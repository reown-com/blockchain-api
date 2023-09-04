use {
    super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, handlers::HistoryQueryParams, state::AppState},
    axum::{
        body::Bytes,
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::abi::Address,
    hyper::HeaderMap,
    std::{net::SocketAddr, sync::Arc},
    tap::TapFallible,
    tracing::{info, log::error},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<HistoryQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    address: Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, path, headers, address, body)
        .with_metrics(HANDLER_TASK_METRICS.with_name("transactions"))
        .await
}

async fn handler_internal(
    state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    query: Query<HistoryQueryParams>,
    _path: MatchedPath,
    _headers: HeaderMap,
    Path(address): Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    address
        .parse::<Address>()
        .map_err(|_| RpcError::IdentityInvalidAddress)?;

    let project = state
        .registry
        .project_data(&query.project_id)
        .await
        .tap_err(|_| state.metrics.add_rejected_project())?;

    project
        .validate_access(&query.project_id, None)
        .tap_err(|e| {
            state.metrics.add_rejected_project();
            info!(
                "Denied access for project: {}, with reason: {}",
                query.project_id, e
            );
        })?;

    let response = state
        .providers
        .history_provider
        .get_transactions(address, body, query.0)
        .await
        .tap_err(|e| {
            error!("Failed to call transaction history with {}", e);
        })?;

    Ok(Json(response).into_response())
}
