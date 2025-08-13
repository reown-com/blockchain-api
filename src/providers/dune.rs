use {
    super::{BalanceProvider, BalanceProviderFactory},
    crate::{
        env::DuneConfig,
        error::{RpcError, RpcResult},
        handlers::balance::{
            BalanceQueryParams, BalanceResponseBody, TokenMetadataCacheItem, H160_EMPTY_ADDRESS,
        },
        providers::{
            balance::{BalanceItem, BalanceQuantity},
            ProviderKind, TokenMetadataCacheProvider,
        },
        utils::{capitalize_first_letter, crypto},
        Metrics,
    },
    async_trait::async_trait,
    deadpool_redis::Pool,
    ethers::types::U256,
    phf::phf_map,
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    tracing::log::error,
    url::Url,
};

const DUNE_API_BASE_URL: &str = "https://api.sim.dune.com";
const MIN_POOL_SIZE: f64 = 10.0; // Minimum pool size to consider a token valid

/// Native token icons, since Dune doesn't provide them yet
/// TODO: Hardcoding icon urls temporarily until Dune provides them
pub static NATIVE_TOKEN_ICONS: phf::Map<&'static str, &'static str> = phf_map! {
      // Solana
      "SOL" => "https://cdn.jsdelivr.net/gh/trustwallet/assets@master/blockchains/solana/info/logo.png",
};

#[derive(Debug, Serialize, Deserialize)]
struct DuneBalanceResponseBody {
    balances: Vec<Balance>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Balance {
    chain: String,
    chain_id: Option<u64>,
    address: String,
    amount: String,
    decimals: Option<u8>,
    symbol: Option<String>,
    price_usd: Option<f64>,
    value_usd: Option<f64>,
    token_metadata: Option<Metadata>,
    pool_size: Option<f64>,
    low_liquidity: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Metadata {
    logo: String,
}

#[derive(Debug)]
pub struct DuneProvider {
    pub provider_kind: ProviderKind,
    pub api_key: String,
    pub http_client: reqwest::Client,
}

impl DuneProvider {
    async fn send_request(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("X-Sim-Api-Key", self.api_key.clone())
            .send()
            .await
    }

    async fn get_evm_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<DuneBalanceResponseBody> {
        let base = format!("{}/v1/evm/balances/{}", DUNE_API_BASE_URL, &address);
        let mut url = Url::parse(&base).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut().append_pair("metadata", "logo");
        if let Some(chain_id_param) = params.chain_id {
            // Check if it's a CAIP2 chain ID (contains a colon)
            let chain_id = if chain_id_param.contains(':') {
                let (_, chain_id) = crypto::disassemble_caip2(&chain_id_param)
                    .map_err(|_| RpcError::InvalidParameter(chain_id_param))?;
                chain_id
            } else {
                // It's already a plain chain ID, use it as EVM chain ID
                chain_id_param
            };
            url.query_pairs_mut().append_pair("chain_ids", &chain_id);
        }

        let latency_start = SystemTime::now();
        let response = self.send_request(url.clone()).await.map_err(|e| {
            error!("Error sending request to Dune EVM balance API: {e:?}");
            RpcError::BalanceProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("evm_balances".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Dune EVM balance response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::BalanceProviderError);
        }
        Ok(response.json::<DuneBalanceResponseBody>().await?)
    }

    async fn get_solana_balance(
        &self,
        address: String,
        metrics: Arc<Metrics>,
    ) -> RpcResult<DuneBalanceResponseBody> {
        let base = format!("{}/beta/svm/balances/{}", DUNE_API_BASE_URL, &address);
        let mut url = Url::parse(&base).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut()
            .append_pair("exclude_spam_tokens", "true");
        url.query_pairs_mut().append_pair("metadata", "logo");

        let latency_start = SystemTime::now();
        let response = self.send_request(url.clone()).await.map_err(|e| {
            error!("Error sending request to Dune svm balance API: {e:?}");
            RpcError::BalanceProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("solana_balances".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Dune Solana balance response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::BalanceProviderError);
        }
        Ok(response.json::<DuneBalanceResponseBody>().await?)
    }
}

#[async_trait]
impl BalanceProvider for DuneProvider {
    async fn get_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        metadata_cache: &Arc<dyn TokenMetadataCacheProvider>,
        metrics: Arc<Metrics>,
    ) -> RpcResult<BalanceResponseBody> {
        let namespace = params
            .chain_id
            .as_ref()
            .map(|chain_id| {
                crypto::disassemble_caip2(chain_id)
                    .map(|(namespace, _)| namespace)
                    .unwrap_or(crypto::CaipNamespaces::Eip155)
            })
            .unwrap_or(crypto::CaipNamespaces::Eip155);

        let balance_response = match namespace {
            crypto::CaipNamespaces::Eip155 => {
                self.get_evm_balance(address, params, metrics.clone())
                    .await?
            }
            crypto::CaipNamespaces::Solana => {
                self.get_solana_balance(address, metrics.clone()).await?
            }
        };

        let mut balances_vec = Vec::new();
        for f in balance_response.balances {
            // Check for the spam token by checking the pool size
            // and low liquidity flags
            if f.pool_size.is_some_and(|size| size <= MIN_POOL_SIZE)
                || f.low_liquidity.unwrap_or(false)
            {
                continue;
            };

            // Build a CAIP-2 chain ID
            let caip2_chain_id = match f.chain_id {
                Some(cid) => format!("{namespace}:{cid}"),
                None => match namespace {
                    // Use default Mainnet chain IDs if not provided
                    crypto::CaipNamespaces::Eip155 => format!("{namespace}:1"),
                    crypto::CaipNamespaces::Solana => {
                        format!("{namespace}:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp")
                    }
                },
            };

            // Build the CAIP-10 address
            let caip10_token_address_strict = if f.address == "native" {
                match namespace {
                    crypto::CaipNamespaces::Eip155 => {
                        format!("{caip2_chain_id}:{H160_EMPTY_ADDRESS}")
                    }
                    crypto::CaipNamespaces::Solana => {
                        format!("{}:{}", caip2_chain_id, crypto::SOLANA_NATIVE_TOKEN_ADDRESS)
                    }
                }
            } else {
                format!("{}:{}", caip2_chain_id, f.address)
            };

            // Skip if no decimals were provided
            // as a possible spam token
            let Some(decimals) = f.decimals else {
                continue;
            };

            // Force to use zero price if the price is not determined
            // instead of not showing the asset
            let price_usd = f.price_usd.unwrap_or(0.0);

            // Get token metadata from the cache or update it
            // Skip the asset if no cached metadata from other providers were added
            // and the current response metadata is empty as a possible spam token
            let token_metadata = match metadata_cache
                .get_metadata(&caip10_token_address_strict)
                .await
            {
                Ok(Some(cached)) => cached,
                Ok(None) => {
                    // Skip if missing required fields and no such metadata
                    // as a possible spam token
                    let Some(symbol) = f.symbol else {
                        continue;
                    };

                    // Determine name
                    let name = if f.address == "native" {
                        capitalize_first_letter(&f.chain)
                    } else {
                        symbol.clone()
                    };

                    // Determine icon URL
                    let icon_url = match &f.token_metadata {
                        Some(m) if !m.logo.is_empty() => m.logo.clone(),
                        _ if f.address == "native" => {
                            NATIVE_TOKEN_ICONS.get(&symbol).unwrap_or(&"").to_string()
                        }
                        Some(m) => m.logo.clone(),
                        None => continue, // Skip tokens without metadata as possible spam
                    };

                    let new_item = TokenMetadataCacheItem {
                        name: name.clone(),
                        symbol: symbol.clone(),
                        icon_url: icon_url.clone(),
                        decimals,
                    };

                    // Spawn a background task to update the cache without blocking
                    {
                        let metadata_cache = metadata_cache.clone();
                        let address_key = caip10_token_address_strict.clone();
                        let new_item_to_store = new_item.clone();
                        tokio::spawn(async move {
                            if let Err(e) = metadata_cache
                                .set_metadata(&address_key, &new_item_to_store)
                                .await
                            {
                                error!("Failed to update token metadata cache: {e:?}");
                            }
                        });
                    }
                    new_item
                }
                Err(e) => {
                    error!("Error getting token metadata: {e:?}");
                    continue;
                }
            };

            // Construct the final BalanceItem
            let balance_item = BalanceItem {
                name: token_metadata.name,
                symbol: token_metadata.symbol,
                chain_id: Some(caip2_chain_id.clone()),
                address: {
                    // Return None if the address is native (for EIP-155).
                    // For Solana’s “native” token, we return the Solana native token address.
                    if f.address == "native" {
                        match namespace {
                            crypto::CaipNamespaces::Eip155 => None,
                            crypto::CaipNamespaces::Solana => {
                                Some(crypto::SOLANA_NATIVE_TOKEN_ADDRESS.to_string())
                            }
                        }
                    } else {
                        Some(format!("{}:{}", caip2_chain_id, f.address))
                    }
                },
                value: f.value_usd,
                price: price_usd,
                quantity: BalanceQuantity {
                    decimals: decimals.to_string(),
                    numeric: crypto::format_token_amount(
                        U256::from_dec_str(&f.amount).unwrap_or_default(),
                        decimals,
                    ),
                },
                icon_url: token_metadata.icon_url,
            };

            balances_vec.push(balance_item);
        }

        Ok(BalanceResponseBody {
            balances: balances_vec,
        })
    }

    fn provider_kind(&self) -> ProviderKind {
        self.provider_kind
    }
}

impl BalanceProviderFactory<DuneConfig> for DuneProvider {
    fn new(provider_config: &DuneConfig, _cache: Option<Arc<Pool>>) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            provider_kind: ProviderKind::Dune,
            api_key: provider_config.api_key.clone(),
            http_client,
        }
    }
}
