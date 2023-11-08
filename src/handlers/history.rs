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
    tracing::log::error,
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

    state.validate_project_access(&query.project_id).await?;

    state.metrics.add_history_lookup();
    let latency_tracker_start = std::time::SystemTime::now();
    let response = state
        .providers
        .history_provider
        .get_transactions(address, body, query.0)
        .await
        .tap_err(|e| {
            error!("Failed to call transaction history with {}", e);
        })?;

    let latency_tracker = latency_tracker_start
        .elapsed()
        .unwrap_or(std::time::Duration::from_secs(0));
    state.metrics.add_history_lookup_success();
    state.metrics.add_history_lookup_latency(latency_tracker);

    Ok(Json(response).into_response())
}
