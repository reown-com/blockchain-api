use {
    crate::handlers::wallet::exchanges::ExchangeType,
    crate::{
        handlers::{SdkInfoParams, HANDLER_TASK_METRICS},
        state::AppState,
    },
    axum::{
        extract::{ConnectInfo, Query, State},
        Json,
    },
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, sync::Arc},
    thiserror::Error,
    tracing::info,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePayUrlRequest {
    pub exchange_id: String,
    pub asset: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePayUrlResponse {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

#[derive(Error, Debug)]
pub enum GetExchangeUrlError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Exchange not found: {0}")]
    ExchangeNotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
    Json(request): Json<GeneratePayUrlRequest>,
) -> Result<GeneratePayUrlResponse, GetExchangeUrlError> {
    handler_internal(state, connect_info, headers, query, request)
        .with_metrics(HANDLER_TASK_METRICS.with_name("pay_get_exchange_url"))
        .await
}

async fn handler_internal(
    state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    _query: Query<QueryParams>,
    request: GeneratePayUrlRequest,
) -> Result<GeneratePayUrlResponse, GetExchangeUrlError> {
    // Get exchange URL
    let exchange = ExchangeType::from_id(&request.exchange_id).ok_or_else(|| {
        GetExchangeUrlError::ExchangeNotFound(format!("Exchange {} not found", request.exchange_id))
    })?;

    let result = exchange
        .get_buy_url(state, &request.asset, &request.amount)
        .await;

    match result {
        Ok(url) => Ok(GeneratePayUrlResponse { url }),
        Err(e) => {
            info!(
                error = %e,
                exchange_id = %request.exchange_id,
                asset = %request.asset,
                amount = %request.amount,
                "Failed to get exchange URL"
            );
            Err(GetExchangeUrlError::InternalError("Unable to get exchange URL".to_string()))
        }
    }
}
