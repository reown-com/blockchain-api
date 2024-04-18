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
pub struct GasPriceQueryParams {
    pub project_id: String,
    pub chain_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GasPriceQueryResponseBody {
    pub standard: String,
    pub fast: String,
    pub instant: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<GasPriceQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("gas_price"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<GasPriceQueryParams>,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query.project_id)
        .await?;

    let response = state
        .providers
        .conversion_provider
        .get_gas_price(query.0)
        .await
        .tap_err(|e| {
            error!("Failed to call get gas price with {}", e);
        })?;

    Ok(Json(response).into_response())
}
