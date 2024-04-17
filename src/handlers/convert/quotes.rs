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
pub struct ConvertQuoteQueryParams {
    pub project_id: String,
    pub amount: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConvertQuoteResponseBody {
    pub quotes: Vec<QuoteItem>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteItem {
    pub id: Option<String>,
    pub from_amount: String,
    pub from_account: String,
    pub to_amount: String,
    pub to_account: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<ConvertQuoteQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("convert_quote"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<ConvertQuoteQueryParams>,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query.project_id)
        .await?;

    let response = state
        .providers
        .conversion_provider
        .get_convert_quote(query.0)
        .await
        .tap_err(|e| {
            error!("Failed to call get conversion quotes with {}", e);
        })?;

    Ok(Json(response).into_response())
}
