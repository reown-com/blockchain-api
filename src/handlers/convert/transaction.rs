use {
    super::super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, state::AppState},
    axum::{
        extract::State,
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
pub struct ConvertTransactionQueryParams {
    pub project_id: String,
    pub amount: String,
    pub from: String,
    pub to: String,
    pub user_address: String,
    pub eip155: Option<ConvertTransactionQueryEip155>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConvertTransactionQueryEip155 {
    pub slippage: usize,
    pub permit: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConvertTransactionResponseBody {
    pub tx: ConvertTx,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConvertTx {
    pub from: String,
    pub to: String,
    pub data: String,
    pub amount: String,
    pub eip155: Option<ConvertTxEip155>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConvertTxEip155 {
    pub gas: String,
    pub gas_price: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    Json(request_payload): Json<ConvertTransactionQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("convert_build_transaction"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    request_payload: ConvertTransactionQueryParams,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&request_payload.project_id)
        .await?;

    let response = state
        .providers
        .conversion_provider
        .build_convert_tx(request_payload)
        .await
        .tap_err(|e| {
            error!("Failed to call build conversion transaction with {}", e);
        })?;

    Ok(Json(response).into_response())
}
