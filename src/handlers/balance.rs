use {
    super::{SdkInfoParams, SupportedCurrencies, HANDLER_TASK_METRICS},
    crate::{
        analytics::{BalanceLookupInfo, MessageSource},
        error::RpcError,
        state::AppState,
        storage::KeyValueStorage,
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
    std::{net::SocketAddr, sync::Arc, time::Duration},
    tap::TapFallible,
    tracing::log::{debug, error},
    wc::future::FutureExt,
};

// Empty address for the contract address mimicking the Ethereum native token
pub const H160_EMPTY_ADDRESS: H160 = H160::repeat_byte(0xee);

const PROVIDER_MAX_CALLS: usize = 2;
const METADATA_CACHE_TTL: Duration = Duration::from_secs(60 * 60 * 24); // 1 day
const BALANCE_CACHE_TTL: Duration = Duration::from_secs(10); // 10 seconds

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct Config {
    /// List of project ids that are not allowed to use the balance RPC call
    /// An empty balances list will be returned for the project ids in the denylist
    pub denylist_project_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQueryParams {
    pub project_id: String,
    pub currency: SupportedCurrencies,
    pub chain_id: Option<String>,
    /// Comma separated list of CAIP-10 contract addresses to force update the balance
    pub force_update: Option<String>,
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadataCacheItem {
    pub name: String,
    pub symbol: String,
    pub icon_url: String,
}

fn token_metadata_cache_key(caip10_token_address: &str) -> String {
    format!("token_metadata/{}", caip10_token_address)
}

fn address_balance_cache_key(address: &str) -> String {
    format!("address_balance/{}", address)
}

pub async fn get_cached_metadata(
    cache: &Option<Arc<dyn KeyValueStorage<TokenMetadataCacheItem>>>,
    caip10_token_address: &str,
) -> Option<TokenMetadataCacheItem> {
    let cache = cache.as_ref()?;
    cache
        .get(&token_metadata_cache_key(caip10_token_address))
        .await
        .unwrap_or(None)
}

pub async fn set_cached_metadata(
    cache: &Option<Arc<dyn KeyValueStorage<TokenMetadataCacheItem>>>,
    caip10_token_address: &str,
    item: &TokenMetadataCacheItem,
) {
    if let Some(cache) = cache {
        cache
            .set(
                &token_metadata_cache_key(caip10_token_address),
                item,
                Some(METADATA_CACHE_TTL),
            )
            .await
            .unwrap_or_else(|e| error!("Failed to set metadata cache: {}", e));
    }
}

pub async fn get_cached_balance(
    cache: &Option<Arc<dyn KeyValueStorage<BalanceResponseBody>>>,
    address: &str,
) -> Option<BalanceResponseBody> {
    let cache = cache.as_ref()?;
    cache
        .get(&address_balance_cache_key(address))
        .await
        .unwrap_or(None)
}

pub async fn set_cached_balance(
    cache: &Option<Arc<dyn KeyValueStorage<BalanceResponseBody>>>,
    address: &str,
    item: &BalanceResponseBody,
) {
    if let Some(cache) = cache {
        cache
            .set(
                &address_balance_cache_key(address),
                item,
                Some(BALANCE_CACHE_TTL),
            )
            .await
            .unwrap_or_else(|e| error!("Failed to set balance cache: {}", e));
    }
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

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<BalanceQueryParams>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Response, RpcError> {
    let project_id = query.project_id.clone();

    // Check the denylist for the project id
    if let Some(denylist_project_ids) = &state.config.balances.denylist_project_ids {
        if denylist_project_ids.contains(&project_id) {
            return Ok(Json(BalanceResponseBody { balances: vec![] }).into_response());
        }
    }

    state.validate_project_access_and_quota(&project_id).await?;

    // if headers not contains `x-sdk-version` and `sv` query parameter then respond
    // with an empty balance array to fix the issue of redundant calls in sdk versions <= 4.1.8
    // https://github.com/WalletConnect/web3modal/pull/2157
    if !headers.contains_key("x-sdk-version") && query.sdk_info.sv.is_none() {
        return Ok(Json(BalanceResponseBody { balances: vec![] }).into_response());
    }

    // Get the cached balance and return it if found except if force_update is needed
    if query.force_update.is_none() {
        if let Some(cached_balance) = get_cached_balance(&state.balance_cache, &address).await {
            return Ok(Json(cached_balance).into_response());
        }
    }

    // If the namespace is not provided, then default to the Ethereum namespace
    let namespace = query
        .chain_id
        .as_ref()
        .map(|chain_id| {
            crypto::disassemble_caip2(chain_id)
                .map(|(namespace, _)| namespace)
                .unwrap_or(crypto::CaipNamespaces::Eip155)
        })
        .unwrap_or(crypto::CaipNamespaces::Eip155);

    if !crypto::is_address_valid(&address, &namespace) {
        return Err(RpcError::InvalidAddress);
    }

    let providers = state
        .providers
        .get_balance_provider_for_namespace(&namespace, PROVIDER_MAX_CALLS)?;

    let mut balance_response = None;
    for provider in providers.iter() {
        let provider_response = provider
            .get_balance(
                address.clone(),
                query.clone().0,
                &state.token_metadata_cache,
                state.metrics.clone(),
            )
            .await
            .tap_err(|e| {
                error!("Failed to call balance with {}", e);
            });

        match provider_response {
            Ok(response) => {
                balance_response = Some((response, provider.provider_kind()));
                break;
            }
            e => {
                debug!("Balance provider returned an error {e:?}, trying the next provider");
            }
        };
    }
    let (mut response, provider_kind) = balance_response.ok_or(
        RpcError::BalanceTemporarilyUnavailable(namespace.to_string()),
    )?;

    {
        let origin = headers
            .get("origin")
            .map(|v| v.to_str().unwrap_or("invalid_header").to_string());

        let (country, continent, region) = state
            .analytics
            .lookup_geo_data(
                network::get_forwarded_ip(&headers).unwrap_or_else(|| connect_info.0.ip()),
            )
            .map(|geo| (geo.country, geo.continent, geo.region))
            .unwrap_or((None, None, None));
        for balance in &response.balances {
            state.analytics.balance_lookup(BalanceLookupInfo::new(
                balance.symbol.clone(),
                balance.chain_id.clone().unwrap_or_default(),
                balance.quantity.numeric.clone(),
                balance.value.unwrap_or(0 as f64),
                balance.price,
                query.currency.to_string(),
                address.clone(),
                project_id.clone(),
                &provider_kind,
                origin.clone(),
                region.clone(),
                country.clone(),
                continent.clone(),
                query.sdk_info.sv.clone(),
                query.sdk_info.st.clone(),
            ));
        }
    }

    // Check for the cache invalidation for the certain token contract addresses and
    // update/override balance results for the token from the RPC call
    if let Some(force_update) = &query.force_update {
        // Force update is only supported on the Ethereum namespace
        if namespace != crypto::CaipNamespaces::Eip155 {
            return Err(RpcError::UnsupportedNamespace(namespace));
        }
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
            let parsed_address = address
                .parse::<Address>()
                .map_err(|_| RpcError::InvalidAddress)?;
            let rpc_balance = crypto::get_erc20_balance(
                &caip2_chain_id,
                contract_address,
                parsed_address,
                rpc_project_id,
                MessageSource::Balance,
                None,
            )
            .await?;
            if let Some(balance) = response
                .balances
                .iter_mut()
                .find(|b| b.address == Some(caip_contract_address.to_string()))
            {
                balance.quantity.numeric = crypto::format_token_amount(
                    rpc_balance,
                    balance.quantity.decimals.parse::<u8>().unwrap_or(0),
                );
                // Recalculating the value with the latest balance
                balance.value = Some(crypto::convert_token_amount_to_value(
                    rpc_balance,
                    balance.price,
                    balance.quantity.decimals.parse::<u8>().unwrap_or(0),
                ));
                continue;
            }
            if contract_address == H160_EMPTY_ADDRESS {
                if let Some(balance) = response
                    .balances
                    .iter_mut()
                    .find(|b| b.address.is_none() && b.chain_id == Some(caip2_chain_id.clone()))
                {
                    balance.quantity.numeric = crypto::format_token_amount(
                        rpc_balance,
                        balance.quantity.decimals.parse::<u8>().unwrap_or(0),
                    );
                    // Recalculate the value with the latest balance
                    balance.value = Some(crypto::convert_token_amount_to_value(
                        rpc_balance,
                        balance.price,
                        balance.quantity.decimals.parse::<u8>().unwrap_or(0),
                    ));
                    continue;
                }
            }
            // Appending the token item to the response if it's not in
            // the balance response due to the zero balance
            let get_price_info_provider = state
                .providers
                .fungible_price_providers
                .get(&namespace)
                .ok_or_else(|| RpcError::UnsupportedNamespace(namespace))?;

            let get_price_info = get_price_info_provider
                .get_price(
                    &chain_id.clone(),
                    format!("{:#x}", contract_address).as_str(),
                    &query.currency,
                    state.metrics.clone(),
                )
                .await
                .tap_err(|e| {
                    error!("Failed to call fungible get_price with {}", e);
                })?;
            let token_info = get_price_info.fungibles.first().ok_or_else(|| {
                error!(
                    "Empty tokens list result from get_price for address: {:#x}",
                    contract_address
                );
                RpcError::BalanceProviderError
            })?;

            response.balances.push(BalanceItem {
                name: token_info.name.clone(),
                symbol: token_info.symbol.clone(),
                chain_id: Some(caip2_chain_id.clone()),
                address: if contract_address == H160_EMPTY_ADDRESS {
                    None
                } else {
                    Some(caip_contract_address.to_string())
                },
                value: Some(crypto::convert_token_amount_to_value(
                    rpc_balance,
                    token_info.price,
                    token_info.decimals,
                )),
                price: token_info.price,
                quantity: BalanceQuantity {
                    decimals: token_info.decimals.to_string(),
                    numeric: crypto::format_token_amount(rpc_balance, token_info.decimals),
                },
                icon_url: token_info.icon_url.clone(),
            });
        }
    }

    // Spawn a background task to update the balance cache without blocking
    {
        tokio::spawn({
            let address_key = address.clone();
            let response = response.clone();
            async move {
                set_cached_balance(&state.balance_cache, &address_key, &response).await;
            }
        });
    }

    Ok(Json(response).into_response())
}
