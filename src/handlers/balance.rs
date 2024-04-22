use {
    super::HANDLER_TASK_METRICS,
    crate::{analytics::BalanceLookupInfo, error::RpcError, state::AppState, utils::network},
    axum::{
        extract::{ConnectInfo, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::abi::Address,
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{
        fmt::Display,
        net::SocketAddr,
        sync::Arc,
        time::{Duration, SystemTime},
    },
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BalanceCurrencies {
    BTC,
    ETH,
    USD,
    EUR,
    GBP,
    AUD,
    CAD,
    INR,
    JPY,
}

impl Display for BalanceCurrencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            BalanceCurrencies::BTC => "btc",
            BalanceCurrencies::ETH => "eth",
            BalanceCurrencies::USD => "usd",
            BalanceCurrencies::EUR => "eur",
            BalanceCurrencies::GBP => "gbp",
            BalanceCurrencies::AUD => "aud",
            BalanceCurrencies::CAD => "cad",
            BalanceCurrencies::INR => "inr",
            BalanceCurrencies::JPY => "jpy",
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQueryParams {
    pub project_id: String,
    pub currency: BalanceCurrencies,
    pub chain_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponseBody {
    pub balances: Vec<BalanceItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceItem {
    pub name: String,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    pub price: f64,
    pub quantity: BalanceQuantity,
    pub icon_url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQuantity {
    pub decimals: String,
    pub numeric: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<BalanceQueryParams>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    address: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, query, connect_info, headers, address)
        .with_metrics(HANDLER_TASK_METRICS.with_name("balance"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<BalanceQueryParams>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Response, RpcError> {
    let project_id = query.project_id.clone();
    address
        .parse::<Address>()
        .map_err(|_| RpcError::InvalidAddress)?;

    state.validate_project_access_and_quota(&project_id).await?;

    // if headers not contains `x-sdk-version` then respond with an empty balance
    // array to fix the issue of redundant calls in sdk versions <= 4.1.8
    // https://github.com/WalletConnect/web3modal/pull/2157
    if !headers.contains_key("x-sdk-version") {
        return Ok(Json(BalanceResponseBody { balances: vec![] }).into_response());
    }

    let start = SystemTime::now();
    let response = state
        .providers
        .balance_provider
        .get_balance(address.clone(), query.clone().0, state.http_client.clone())
        .await
        .tap_err(|e| {
            error!("Failed to call balance with {}", e);
        })?;
    let latency = start.elapsed().unwrap_or(Duration::from_secs(0));

    {
        let origin = headers
            .get("origin")
            .map(|v| v.to_str().unwrap_or("invalid_header").to_string());

        let (country, continent, region) = state
            .analytics
            .lookup_geo_data(
                network::get_forwarded_ip(headers).unwrap_or_else(|| connect_info.0.ip()),
            )
            .map(|geo| (geo.country, geo.continent, geo.region))
            .unwrap_or((None, None, None));
        for balance in &response.balances {
            state.analytics.balance_lookup(BalanceLookupInfo::new(
                latency,
                balance.symbol.clone(),
                balance.chain_id.clone().unwrap_or_default(),
                balance.quantity.numeric.clone(),
                balance.value.unwrap_or(0 as f64),
                balance.price,
                query.currency.to_string(),
                address.clone(),
                project_id.clone(),
                origin.clone(),
                region.clone(),
                country.clone(),
                continent.clone(),
            ));
        }
    }

    Ok(Json(response).into_response())
}
