use {
    super::{RpcQueryParams, HANDLER_TASK_METRICS},
    crate::{error::RpcError, state::AppState},
    axum::{
        extract::{Query, State},
        response::Response,
    },
    axum_tungstenite::WebSocketUpgrade,
    std::sync::Arc,
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<RpcQueryParams>,
    ws: WebSocketUpgrade,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params, ws)
        .with_metrics(HANDLER_TASK_METRICS.with_name("ws_proxy"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
    Query(query_params): Query<RpcQueryParams>,
    ws: WebSocketUpgrade,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query_params.project_id)
        .await?;

    let chain_id = query_params.chain_id.to_lowercase();
    let provider = state
        .providers
        .get_ws_provider_for_chain_id(&chain_id)
        .ok_or(RpcError::UnsupportedChain(chain_id.clone()))?;

    state.metrics.add_websocket_connection(chain_id);

    provider.proxy(ws, query_params).await
}
