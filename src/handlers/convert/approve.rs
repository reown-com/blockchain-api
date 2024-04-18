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
pub struct ConvertApproveQueryParams {
    pub project_id: String,
    pub from: String,
    pub to: String,
    pub amount: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConvertApproveResponseBody {
    pub tx: ConvertApproveTx,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConvertApproveTx {
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: String,
    pub eip155: Option<ConvertApproveTxEip155>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConvertApproveTxEip155 {
    pub gas_price: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<ConvertApproveQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("convert_approve_tx"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<ConvertApproveQueryParams>,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query.project_id)
        .await?;

    let response = state
        .providers
        .conversion_provider
        .build_approve_tx(query.0)
        .await
        .tap_err(|e| {
            error!("Failed to call build approve tx for conversion with {}", e);
        })?;

    Ok(Json(response).into_response())
}
