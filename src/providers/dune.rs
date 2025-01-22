use {
    super::{BalanceProvider, BalanceProviderFactory},
    crate::{
        env::DuneConfig,
        error::{RpcError, RpcResult},
        handlers::balance::{
            get_cached_metadata, set_cached_metadata, BalanceQueryParams, BalanceResponseBody,
            TokenMetadataCacheItem, H160_EMPTY_ADDRESS,
        },
        providers::{
            balance::{BalanceItem, BalanceQuantity},
            ProviderKind,
        },
        storage::KeyValueStorage,
        utils::crypto,
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

const DUNE_API_BASE_URL: &str = "https://api.dune.com/api";

/// Native token icons, since Dune doesn't provide them yet
/// TODO: Hardcoding icon urls temporarily until Dune provides them
pub static NATIVE_TOKEN_ICONS: phf::Map<&'static str, &'static str> = phf_map! {
      // Ethereum
      "ETH" => "https://cdn.jsdelivr.net/gh/trustwallet/assets@master/blockchains/ethereum/info/logo.png",
      // Polygon
      "POL" => "https://cdn.jsdelivr.net/gh/trustwallet/assets@master/blockchains/polygon/info/logo.png",
      // xDAI
      "XDAI" => "https://cdn.jsdelivr.net/gh/trustwallet/assets@master/blockchains/xdai/info/logo.png",
      // BNB
      "BNB" => "https://cdn.jsdelivr.net/gh/trustwallet/assets@master/blockchains/binance/info/logo.png",
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
            .header("X-Dune-Api-Key", self.api_key.clone())
            .send()
            .await
    }

    async fn get_evm_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<DuneBalanceResponseBody> {
        let base = format!("{}/echo/v1/balances/evm/{}", DUNE_API_BASE_URL, &address);
        let mut url = Url::parse(&base).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut()
            .append_pair("exclude_spam_tokens", "true");
        url.query_pairs_mut().append_pair("metadata", "logo");
        if let Some(caip2_chain_id) = params.chain_id {
            let (_, chain_id) = crypto::disassemble_caip2(&caip2_chain_id)
                .map_err(|_| RpcError::InvalidParameter(caip2_chain_id))?;
            url.query_pairs_mut().append_pair("chain_ids", &chain_id);
        }

        let latency_start = SystemTime::now();
        let response = self.send_request(url.clone()).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
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
        let base = format!("{}/echo/beta/balances/svm/{}", DUNE_API_BASE_URL, &address);
        let mut url = Url::parse(&base).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut()
            .append_pair("exclude_spam_tokens", "true");
        url.query_pairs_mut().append_pair("metadata", "logo");

        let latency_start = SystemTime::now();
        let response = self.send_request(url.clone()).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
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
    #[tracing::instrument(skip(self, params), fields(provider = "Dune"), level = "debug")]
    async fn get_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        metadata_cache: &Option<Arc<dyn KeyValueStorage<TokenMetadataCacheItem>>>,
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
            // Build a CAIP-2 chain ID
            let caip2_chain_id = match f.chain_id {
                Some(cid) => format!("{}:{}", namespace, cid),
                None => match namespace {
                    // Use default Mainnet chain IDs if not provided
                    crypto::CaipNamespaces::Eip155 => format!("{}:1", namespace),
                    crypto::CaipNamespaces::Solana => {
                        format!("{}:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", namespace)
                    }
                },
            };

            // Build the CAIP-10 address
            let caip10_token_address_strict = if f.address == "native" {
                match namespace {
                    crypto::CaipNamespaces::Eip155 => {
                        format!("{}:{}", caip2_chain_id, H160_EMPTY_ADDRESS)
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
            let token_metadata =
                match get_cached_metadata(metadata_cache, &caip10_token_address_strict).await {
                    Some(cached) => cached,
                    None => {
                        // Skip if missing required fields and no such metadata
                        // as a possible spam token
                        let Some(symbol) = f.symbol else {
                            continue;
                        };

                        // Determine name
                        let name = if f.address == "native" {
                            f.chain.clone()
                        } else {
                            symbol.clone()
                        };

                        // Determine icon URL
                        let icon_url = if f.address == "native" {
                            NATIVE_TOKEN_ICONS.get(&symbol).unwrap_or(&"").to_string()
                        } else {
                            // If there's no token_metadata or no logo, skip the asset
                            // as a possible spam token
                            match &f.token_metadata {
                                Some(m) => m.logo.clone(),
                                None => continue,
                            }
                        };

                        let new_item = TokenMetadataCacheItem {
                            name: name.clone(),
                            symbol: symbol.clone(),
                            icon_url: icon_url.clone(),
                        };

                        // Spawn a background task to update the cache without blocking
                        {
                            let metadata_cache = metadata_cache.clone();
                            let address_key = caip10_token_address_strict.clone();
                            let new_item_to_store = new_item.clone();
                            tokio::spawn(async move {
                                set_cached_metadata(
                                    &metadata_cache,
                                    &address_key,
                                    &new_item_to_store,
                                )
                                .await;
                            });
                        }
                        new_item
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
