use {
    super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, state::AppState},
    axum::{
        extract::State,
        response::{IntoResponse, Response},
        Json,
    },
    hyper::header::CACHE_CONTROL,
    std::sync::Arc,
    wc::future::FutureExt,
};

pub async fn handler(state: State<Arc<AppState>>) -> Result<Response, RpcError> {
    handler_internal(state)
        .with_metrics(HANDLER_TASK_METRICS.with_name("supported_chains"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(State(state): State<Arc<AppState>>) -> Result<Response, RpcError> {
    // Set cache control headers to 24 hours
    let ttl_secs = 24 * 60 * 60;

    Ok((
        [(
            CACHE_CONTROL,
            format!("public, max-age={ttl_secs}, s-maxage={ttl_secs}"),
        )],
        Json(state.providers.rpc_supported_chains.clone()),
    )
        .into_response())
}
