use {
    crate::{error::RpcError, handlers::HANDLER_TASK_METRICS, state::AppState},
    axum::{
        extract::{ConnectInfo, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, sync::Arc},
    tap::TapFallible,
    tracing::log::error,
    validator::Validate,
    wc::future::FutureExt,
};

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct OnRampBuyQuotesParams {
    #[serde(rename(deserialize = "projectId"))]
    pub project_id: String,
    #[validate(length(max = 4))]
    #[serde(rename(deserialize = "purchaseCurrency"))]
    pub purchase_currency: String,
    #[serde(rename(deserialize = "purchaseNetwork"))]
    pub purchase_network: Option<String>,
    #[serde(rename(deserialize = "paymentAmount"))]
    pub payment_amount: String,
    #[validate(length(max = 4))]
    #[serde(rename(deserialize = "paymentCurrency"))]
    pub payment_currency: String,
    #[serde(rename(deserialize = "paymentMethod"))]
    pub payment_method: String,
    #[validate(length(equal = 2))]
    #[serde(rename(deserialize = "country"))]
    pub country: String,
    #[validate(length(equal = 2))]
    #[serde(rename(deserialize = "subdivision"))]
    pub subdivision: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnRampBuyQuotesResponse {
    #[serde(rename(serialize = "paymentTotal"))]
    pub payment_total: PayOptionValue,
    #[serde(rename(serialize = "paymentSubTotal"))]
    pub payment_subtotal: PayOptionValue,
    #[serde(rename(serialize = "purchaseAmount"))]
    pub purchase_amount: PayOptionValue,
    #[serde(rename(serialize = "coinbaseFee"))]
    pub coinbase_fee: PayOptionValue,
    #[serde(rename(serialize = "networkFee"))]
    pub network_fee: PayOptionValue,
    #[serde(rename(serialize = "quoteId"))]
    pub quote_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayOptionValue {
    pub value: String,
    pub currency: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<OnRampBuyQuotesParams>,
    headers: HeaderMap,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, headers)
        .with_metrics(HANDLER_TASK_METRICS.with_name("onrmap_buy_quotes"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    query: Query<OnRampBuyQuotesParams>,
    _headers: HeaderMap,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query.project_id)
        .await?;

    let buy_quotes = state
        .providers
        .onramp_provider
        .get_buy_quotes(query.0, state.http_client.clone())
        .await
        .tap_err(|e| {
            error!("Failed to call coinbase buy quotes with {}", e);
        })?;

    Ok(Json(buy_quotes).into_response())
}
