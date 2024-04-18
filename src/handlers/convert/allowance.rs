use {
    super::super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, state::AppState},
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllowanceQueryParams {
    pub project_id: String,
    pub token_address: String,
    pub user_address: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllowanceResponseBody {
    pub allowance: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<AllowanceQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("conversion_allowance"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<AllowanceQueryParams>,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query.project_id)
        .await?;

    let response = state
        .providers
        .conversion_provider
        .get_allowance(query.0)
        .await
        .tap_err(|e| {
            error!("Failed to call get allownce with {}", e);
        })?;

    Ok(Json(response).into_response())
}
