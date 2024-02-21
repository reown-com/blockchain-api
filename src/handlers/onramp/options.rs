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

#[derive(Debug, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct OnRampBuyOptionsParams {
    pub project_id: String,
    #[validate(length(equal = 2))]
    pub country: String,
    #[validate(length(equal = 2))]
    pub subdivision: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnRampBuyOptionsResponse {
    #[serde(rename(serialize = "paymentCurrencies"))]
    pub payment_currencies: Vec<PaymentCurrenciesLimits>,
    #[serde(rename(serialize = "purchaseCurrencies"))]
    pub purchase_currencies: Vec<PurchaseCurrencies>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentCurrenciesLimits {
    pub id: String,
    pub limits: Vec<PaymentCurrenciesLimit>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentCurrenciesLimit {
    pub id: String,
    pub min: String,
    pub max: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurchaseCurrencies {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub networks: Vec<PurchaseCurrencyNetwork>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PurchaseCurrencyNetwork {
    pub name: String,
    #[serde(rename(serialize = "displayName"))]
    pub display_name: String,
    #[serde(rename(serialize = "contractAddress"))]
    pub contract_address: String,
    #[serde(rename(serialize = "chainId"))]
    pub chain_id: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<OnRampBuyOptionsParams>,
    headers: HeaderMap,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, headers)
        .with_metrics(HANDLER_TASK_METRICS.with_name("onrmap_buy_options"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    query: Query<OnRampBuyOptionsParams>,
    _headers: HeaderMap,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query.project_id)
        .await?;

    let buy_options = state
        .providers
        .onramp_provider
        .get_buy_options(query.0, state.http_client.clone())
        .await
        .tap_err(|e| {
            error!("Failed to call coinbase buy options with {}", e);
        })?;

    Ok(Json(buy_options).into_response())
}
