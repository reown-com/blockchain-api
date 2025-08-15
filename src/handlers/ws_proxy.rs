use {
    super::{RpcQueryParams, HANDLER_TASK_METRICS},
    crate::{error::RpcError, state::AppState},
    axum::{
        extract::{ws::WebSocketUpgrade, Query, State},
        http::HeaderMap,
        response::Response,
    },
    std::sync::Arc,
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<RpcQueryParams>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params, headers, ws)
        .with_metrics(HANDLER_TASK_METRICS.with_name("ws_proxy"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
    Query(query_params): Query<RpcQueryParams>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, RpcError> {
    // Check if this is actually a WebSocket connection
    if !is_websocket_request(&headers) {
        return Err(RpcError::WebSocketConnectionExpected);
    }
    state
        .validate_project_access_and_quota(&query_params.project_id)
        .await?;

    let chain_id = query_params.chain_id.clone();
    let provider = state
        .providers
        .get_ws_provider_for_chain_id(&chain_id)
        .ok_or(RpcError::UnsupportedChain(chain_id.clone()))?;

    state.metrics.add_websocket_connection(chain_id);

    provider.proxy(ws, query_params).await
}

/// Check if the request is a WebSocket upgrade request
fn is_websocket_request(headers: &HeaderMap) -> bool {
    let has_upgrade_header = headers
        .get("upgrade")
        .map(|v| v.as_bytes().eq_ignore_ascii_case(b"websocket"))
        .unwrap_or(false);

    let has_connection_header = headers
        .get("connection")
        .map(|v| v.as_bytes().eq_ignore_ascii_case(b"upgrade"))
        .unwrap_or(false);

    has_upgrade_header && has_connection_header
}
