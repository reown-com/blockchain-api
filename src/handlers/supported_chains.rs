use {
    super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, providers::SupportedChains, state::AppState},
    axum::{extract::State, Json},
    std::sync::Arc,
    wc::future::FutureExt,
};

pub async fn handler(state: State<Arc<AppState>>) -> Result<Json<SupportedChains>, RpcError> {
    handler_internal(state)
        .with_metrics(HANDLER_TASK_METRICS.with_name("supported_chains"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SupportedChains>, RpcError> {
    Ok(Json(state.providers.supported_chains.clone()))
}
