use {
    super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, state::AppState},
    axum::{extract::State, Json},
    std::{collections::HashSet, sync::Arc},
    wc::future::FutureExt,
};

pub async fn handler(state: State<Arc<AppState>>) -> Result<Json<HashSet<String>>, RpcError> {
    handler_internal(state)
        .with_metrics(HANDLER_TASK_METRICS.with_name("supported_chains"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HashSet<String>>, RpcError> {
    Ok(Json(state.providers.supported_chains.clone()))
}
