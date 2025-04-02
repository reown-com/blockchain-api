use {
    crate::{
        error::RpcError, handlers::HANDLER_TASK_METRICS, state::AppState,
        utils::simple_request_json::SimpleRequestJson,
    },
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub project_id: String,
    pub country_code: Option<String>,
    pub destination_currency_code: String,
    pub payment_method_type: Option<String>,
    pub source_amount: f64,
    pub source_currency_code: String,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuotesResponse {
    pub country_code: Option<String>,
    pub customer_score: Option<f64>,
    pub destination_amount: f64,
    pub destination_amount_without_fees: Option<f64>,
    pub destination_currency_code: String,
    pub exchange_rate: Option<f64>,
    pub fiat_amount_without_fees: Option<f64>,
    pub low_kyc: Option<bool>,
    pub network_fee: Option<f64>,
    pub payment_method_type: Option<String>,
    pub service_provider: Option<String>,
    pub source_amount: f64,
    pub source_amount_without_fees: Option<f64>,
    pub source_currency_code: Option<String>,
    pub total_fee: Option<f64>,
    pub transaction_fee: Option<f64>,
    pub transaction_type: Option<String>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    SimpleRequestJson(request_payload): SimpleRequestJson<QueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("onramp_multiproviders_quotes"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    request_payload: QueryParams,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&request_payload.project_id)
        .await?;

    let quotes = state
        .providers
        .onramp_multi_provider
        .get_quotes(request_payload, state.metrics.clone())
        .await
        .tap_err(|e| {
            error!("Failed to call onramp multi providers quotes with {}", e);
        })?;

    Ok(Json(quotes).into_response())
}
