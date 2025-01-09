use {
    super::{BalanceProvider, BalanceProviderFactory},
    crate::{
        env::DuneConfig,
        error::{RpcError, RpcResult},
        handlers::balance::{BalanceQueryParams, BalanceResponseBody},
        providers::{
            balance::{BalanceItem, BalanceQuantity},
            ProviderKind,
        },
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

/// Native token icons, since Dune doesn't provide them yet
/// TODO: Hardcoding icon urls temporarily until Dune provides them
pub static NATIVE_TOKEN_ICONS: phf::Map<&'static str, &'static str> = phf_map! {
      // Ethereum
      "ETH" => "https://assets.coingecko.com/coins/images/279/small/ethereum.png",
      // Polygon
      "POL" => "https://assets.coingecko.com/coins/images/32440/standard/polygon.png",
      // xDAI
      "XDAI" => "https://assets.coingecko.com/coins/images/662/standard/logo_square_simple_300px.png",
      // BNB
      "BNB" => "https://assets.coingecko.com/coins/images/825/small/binance-coin-logo.png",
};

#[derive(Debug, Serialize, Deserialize)]
struct DuneResponseBody {
    balances: Vec<Balance>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Balance {
    chain: String,
    chain_id: u64,
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
}

#[async_trait]
impl BalanceProvider for DuneProvider {
    #[tracing::instrument(skip(self, params), fields(provider = "Dune"), level = "debug")]
    async fn get_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<BalanceResponseBody> {
        let base = format!("https://api.dune.com/api/echo/v1/balances/evm/{}", &address);
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
            Some("balances".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Dune balance response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::BalanceProviderError);
        }
        let body = response.json::<DuneResponseBody>().await?;

        let balances_vec = body
            .balances
            .into_iter()
            .filter_map(|mut f| {
                // Skip the asset if there are no symbol, decimals, since this
                // is likely a spam token
                let symbol = f.symbol.take()?;
                let price_usd = f.price_usd.take()?;
                let decimals = f.decimals.take()?;
                let caip2_chain_id = format!("eip155:{}", f.chain_id.clone());
                Some(BalanceItem {
                    name: {
                        if f.address == "native" {
                            f.chain
                        } else {
                            symbol.clone()
                        }
                    },
                    symbol: symbol.clone(),
                    chain_id: Some(caip2_chain_id),
                    address: {
                        // Return None if the address is native for the native token
                        if f.address == "native" {
                            None
                        } else {
                            Some(format!("eip155:{}:{}", f.chain_id, f.address.clone()))
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
                    icon_url: {
                        if f.address == "native" {
                            NATIVE_TOKEN_ICONS.get(&symbol).unwrap_or(&"").to_string()
                        } else {
                            f.token_metadata?.logo
                        }
                    },
                })
            })
            .collect::<Vec<_>>();

        let response = BalanceResponseBody {
            balances: balances_vec,
        };

        Ok(response)
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
