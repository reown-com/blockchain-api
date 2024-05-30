use {
    super::HANDLER_TASK_METRICS,
    crate::{
        analytics::BalanceLookupInfo,
        error::RpcError,
        state::AppState,
        utils::{crypto, network},
    },
    axum::{
        extract::{ConnectInfo, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::{abi::Address, types::H160},
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{
        fmt::Display,
        net::SocketAddr,
        sync::Arc,
        time::{Duration, SystemTime},
    },
    tap::TapFallible,
    tracing::log::{debug, error},
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
        write!(
            f,
            "{}",
            match self {
                BalanceCurrencies::BTC => "btc",
                BalanceCurrencies::ETH => "eth",
                BalanceCurrencies::USD => "usd",
                BalanceCurrencies::EUR => "eur",
                BalanceCurrencies::GBP => "gbp",
                BalanceCurrencies::AUD => "aud",
                BalanceCurrencies::CAD => "cad",
                BalanceCurrencies::INR => "inr",
                BalanceCurrencies::JPY => "jpy",
            }
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQueryParams {
    pub project_id: String,
    pub currency: BalanceCurrencies,
    pub chain_id: Option<String>,
    /// Comma separated list of CAIP-10 contract addresses to force update the balance
    pub force_update: Option<String>,
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
    let parsed_address = address
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
    let mut response = state
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

    // Check for the cache invalidation for the certain token contract addresses and
    // update/override balance results for the token from the RPC call
    if let Some(force_update) = &query.force_update {
        let rpc_project_id = state
            .config
            .server
            .testing_project_id
            .as_ref()
            .ok_or_else(|| {
                RpcError::InvalidConfiguration(
                    "Missing testing project id in the configuration for the balance RPC lookups"
                        .to_string(),
                )
            })?;
        let force_update: Vec<&str> = force_update.split(',').collect();
        for caip_contract_address in force_update {
            debug!(
                "Forcing balance update for the contract address: {}",
                caip_contract_address
            );
            let (namespace, chain_id, contract_address) =
                crypto::disassemble_caip10(caip_contract_address)
                    .map_err(|_| RpcError::InvalidAddress)?;
            let contract_address = contract_address
                .parse::<Address>()
                .map_err(|_| RpcError::InvalidAddress)?;
            let caip2_chain_id = format!("{}:{}", namespace, chain_id);
            let rpc_balance = crypto::get_erc20_balance(
                &caip2_chain_id,
                contract_address,
                parsed_address,
                rpc_project_id,
                "balance",
            )
            .await?;
            if let Some(balance) = response
                .balances
                .iter_mut()
                .find(|b| b.address == Some(caip_contract_address.to_string()))
            {
                balance.quantity.numeric = crypto::format_token_amount(
                    rpc_balance,
                    balance.quantity.decimals.parse::<u32>().unwrap_or(0),
                );
                // Recalculating the value with the latest balance
                balance.value = Some(crypto::convert_token_amount_to_value(
                    rpc_balance,
                    balance.price,
                    balance.quantity.decimals.parse::<u32>().unwrap_or(0),
                ));
            }
            if contract_address == H160::repeat_byte(0xee) {
                if let Some(balance) = response
                    .balances
                    .iter_mut()
                    .find(|b| b.address.is_none() && b.chain_id == Some(caip2_chain_id.clone()))
                {
                    balance.quantity.numeric = crypto::format_token_amount(
                        rpc_balance,
                        balance.quantity.decimals.parse::<u32>().unwrap_or(0),
                    );
                    // Recalculate the value with the latest balance
                    balance.value = Some(crypto::convert_token_amount_to_value(
                        rpc_balance,
                        balance.price,
                        balance.quantity.decimals.parse::<u32>().unwrap_or(0),
                    ));
                }
            }
        }
    }

    Ok(Json(response).into_response())
}
