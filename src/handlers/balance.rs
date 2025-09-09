use {
    super::{SdkInfoParams, SupportedCurrencies},
    crate::{
        analytics::{BalanceLookupInfo, MessageSource},
        error::RpcError,
        providers::TokenMetadataCacheProvider,
        state::AppState,
        storage::{error::StorageError, KeyValueStorage},
        utils::{crypto, network},
    },
    async_trait::async_trait,
    axum::{
        extract::{ConnectInfo, Path, Query, State},
        Json,
    },
    deadpool_redis::{redis::AsyncCommands, Pool},
    ethers::{abi::Address, types::H160},
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, sync::Arc, time::Duration},
    tap::TapFallible,
    tracing::log::{debug, error},
    wc::metrics::{future_metrics, FutureExt},
};

// Empty address for the contract address mimicking the Ethereum native token
pub const H160_EMPTY_ADDRESS: H160 = H160::repeat_byte(0xee);

const PROVIDER_MAX_CALLS: usize = 2;
const METADATA_CACHE_TTL: u64 = 60 * 60 * 24; // 1 day
const BALANCE_CACHE_TTL: Duration = Duration::from_secs(10); // 10 seconds

// List of SDK versions that should return an empty balance response
// to fix the issue of redundant calls in SDK versions
const EMPTY_BALANCE_RESPONSE_SDK_VERSIONS: [&str; 2] = ["1.6.4", "1.6.5"];

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
    pub decimals: u8,
}

fn address_balance_cache_key(address: &str) -> String {
    format!("address_balance/{address}")
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
            .unwrap_or_else(|e| error!("Failed to set balance cache: {e}"));
    }
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<BalanceQueryParams>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    address: Path<String>,
) -> Result<Json<BalanceResponseBody>, RpcError> {
    handler_internal(state, query, connect_info, headers, address)
        .with_metrics(future_metrics!("handler:balance"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<BalanceQueryParams>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Json<BalanceResponseBody>, RpcError> {
    let project_id = query.project_id.clone();

    // Check the denylist for the project id
    if let Some(denylist_project_ids) = &state.config.balances.denylist_project_ids {
        if denylist_project_ids.contains(&project_id) {
            return Ok(Json(BalanceResponseBody { balances: vec![] }));
        }
    }

    // Check if `origin` is empty and return empty balance response in this case
    let origin = headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if origin.is_empty() {
        debug!("Origin is empty, returning empty balance response");
        return Ok(Json(BalanceResponseBody { balances: vec![] }));
    }

    state.validate_project_access_and_quota(&project_id).await?;

    // if headers not contains `x-sdk-version` and `sv` query parameter then respond
    // with an empty balance array to fix the issue of redundant calls in sdk versions <= 4.1.8
    // https://github.com/WalletConnect/web3modal/pull/2157
    if !headers.contains_key("x-sdk-version") && query.sdk_info.sv.is_none() {
        return Ok(Json(BalanceResponseBody { balances: vec![] }));
    }

    let sdk_version = query
        .sdk_info
        .sv
        .as_deref()
        .or_else(|| headers.get("x-sdk-version").and_then(|v| v.to_str().ok()));

    // Respond with an empty balance array if the sdk version is in the empty balance response list
    // because the sdk version has a bug that causes excessive amount of calls to the balance RPC
    if let Some(version) = sdk_version {
        for &v in &EMPTY_BALANCE_RESPONSE_SDK_VERSIONS {
            if version == v || version.ends_with(v) {
                debug!("Responding with an empty balance array for sdk version: {version}");
                return Ok(Json(BalanceResponseBody { balances: vec![] }));
            }
        }
    }

    // Get the cached balance and return it if found except if force_update is needed
    if query.force_update.is_none() {
        if let Some(cached_balance) = get_cached_balance(&state.balance_cache, &address).await {
            return Ok(Json(cached_balance));
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
    let mut retry_count = 0;
    for (i, provider) in providers.iter().enumerate() {
        let provider_response = provider
            .get_balance(
                address.clone(),
                query.clone().0,
                &state.providers.token_metadata_cache,
                state.metrics.clone(),
            )
            .await;
        match provider_response {
            Ok(response) => {
                balance_response = Some((response, provider.provider_kind()));
                break;
            }
            Err(e) => {
                retry_count = i;
                error!("Error on balance provider response, trying the next provider: {e:?}");
            }
        };
    }
    state
        .metrics
        .add_balance_lookup_retries(retry_count as u64, namespace);

    let (mut response, provider_kind) = balance_response.ok_or(
        RpcError::BalanceTemporarilyUnavailable(namespace.to_string()),
    )?;

    {
        // Filling the request_id from the `propagate_x_request_id` middleware
        let request_id = headers
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown");
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
                request_id.to_string(),
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
            debug!("Forcing balance update for the contract address: {caip_contract_address}");
            let (namespace, chain_id, contract_address) =
                crypto::disassemble_caip10(caip_contract_address)
                    .map_err(|_| RpcError::InvalidAddress)?;
            let contract_address = contract_address
                .parse::<Address>()
                .map_err(|_| RpcError::InvalidAddress)?;
            let caip2_chain_id = format!("{namespace}:{chain_id}");
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
                    format!("{contract_address:#x}").as_str(),
                    &query.currency,
                    &state.providers.token_metadata_cache,
                    state.metrics.clone(),
                )
                .await
                .tap_err(|e| {
                    error!("Failed to call fungible get_price with {e}");
                })?;
            let token_info = get_price_info.fungibles.first().ok_or_else(|| {
                error!(
                    "Empty tokens list result from get_price for address: {contract_address:#x}"
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
    Ok(Json(response))
}

pub struct TokenMetadataCache {
    cache_pool: Option<Arc<Pool>>,
}

impl TokenMetadataCache {
    pub fn new(cache_pool: Option<Arc<Pool>>) -> Self {
        Self { cache_pool }
    }
    fn token_metadata_cache_key(&self, caip10_token_address: &str) -> String {
        format!("token_metadata/{caip10_token_address}")
    }

    #[allow(dependency_on_unit_never_type_fallback)]
    async fn set_cache(&self, key: &str, value: &str, ttl: u64) -> Result<(), StorageError> {
        if let Some(redis_pool) = &self.cache_pool {
            let mut cache = redis_pool.get().await.map_err(|e| {
                StorageError::Connection(format!("Error when getting the Redis pool instance {e}"))
            })?;
            cache
                .set_ex(key, value, ttl)
                .await
                .map_err(|e| StorageError::Connection(format!("Error when seting cache: {e}")))?;
        }
        Ok(())
    }

    #[allow(dependency_on_unit_never_type_fallback)]
    async fn get_cache(&self, key: &str) -> Result<Option<String>, StorageError> {
        if let Some(redis_pool) = &self.cache_pool {
            let mut cache = redis_pool.get().await.map_err(|e| {
                StorageError::Connection(format!("Error when getting the Redis pool instance {e}"))
            })?;
            let value = cache
                .get(key)
                .await
                .map_err(|e| StorageError::Connection(format!("Error when getting cache: {e}")))?;
            return Ok(value);
        }
        Ok(None)
    }
}

#[async_trait]
impl TokenMetadataCacheProvider for TokenMetadataCache {
    async fn get_metadata(
        &self,
        caip10_token_address: &str,
    ) -> Result<Option<TokenMetadataCacheItem>, RpcError> {
        if let Some(redis_pool) = self
            .get_cache(&self.token_metadata_cache_key(caip10_token_address))
            .await?
        {
            let metadata: TokenMetadataCacheItem = serde_json::from_str(&redis_pool)?;
            return Ok(Some(metadata));
        }
        Ok(None)
    }

    async fn set_metadata(
        &self,
        caip10_token_address: &str,
        item: &TokenMetadataCacheItem,
    ) -> Result<(), RpcError> {
        self.set_cache(
            &self.token_metadata_cache_key(caip10_token_address),
            &serde_json::to_string(&item)?,
            METADATA_CACHE_TTL,
        )
        .await?;
        Ok(())
    }
}
