use {
    crate::{
        error::{RpcError, RpcResult},
        handlers::{fungible_price::FungiblePriceItem, SupportedCurrencies},
        providers::{
            FungiblePriceProvider, PriceResponseBody, ProviderKind, TokenMetadataCacheProvider,
        },
        utils::crypto,
        Metrics,
    },
    async_trait::async_trait,
    serde::Deserialize,
    std::{sync::Arc, time::SystemTime},
    tracing::log::error,
    url::Url,
};

#[derive(Debug)]
pub struct LifiProvider {
    pub provider_kind: ProviderKind,
    pub api_key: Option<String>,
    pub base_api_url: String,
    pub http_client: reqwest::Client,
}

impl LifiProvider {
    pub fn new(api_key: Option<String>) -> Self {
        let base_api_url = "https://li.quest/v1".to_string();
        let http_client = reqwest::Client::new();
        Self {
            provider_kind: ProviderKind::Lifi,
            api_key,
            base_api_url,
            http_client,
        }
    }

    async fn send_request(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        if let Some(api_key) = &self.api_key {
            self.http_client
                .get(url)
                .header("x-lifi-api-key", api_key.clone())
                .send()
                .await
        } else {
            self.http_client.get(url).send().await
        }
    }

    pub async fn get_token_info(
        &self,
        chain_id: &str,
        address: &str,
        _currency: &SupportedCurrencies,
        metrics: Arc<Metrics>,
    ) -> Result<LifiTokenItem, RpcError> {
        let address = address.to_lowercase();
        let mut url = Url::parse(format!("{}/token", &self.base_api_url).as_str())
            .map_err(|_| RpcError::ConversionParseURLError)?;
        url.query_pairs_mut().append_pair("chain", chain_id);
        url.query_pairs_mut().append_pair("token", &address);

        let latency_start = SystemTime::now();
        let price_response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to Lifi provider for fungible price: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            price_response.status().into(),
            latency_start,
            Some(chain_id.to_string()),
            Some("token_info".to_string()),
        );

        if !price_response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if price_response.status() == reqwest::StatusCode::BAD_REQUEST {
                return Err(RpcError::ConversionInvalidParameter(
                    "Invalid token or chain parameter".to_string(),
                ));
            }
            // 404 response is expected when the asset is not supported
            if price_response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(RpcError::AssetNotSupported(address.clone()));
            }

            error!(
                "Error on getting fungible price from Lifi provider. Status is not OK: {:?}",
                price_response.status(),
            );
            return Err(RpcError::ConversionProviderError);
        }
        let price_body = price_response.json::<LifiTokenItem>().await?;
        Ok(price_body)
    }
}

#[derive(Debug, Deserialize)]
pub struct LifiTokenItem {
    symbol: String,
    name: String,
    decimals: u8,
    #[serde(alias = "logoURI")]
    logo_uri: Option<String>,
    #[serde(alias = "priceUSD")]
    price_usd: Option<String>,
}

#[async_trait]
impl FungiblePriceProvider for LifiProvider {
    async fn get_price(
        &self,
        chain_id: &str,
        address: &str,
        currency: &SupportedCurrencies,
        _metadata_cache: &Arc<dyn TokenMetadataCacheProvider>,
        metrics: Arc<Metrics>,
    ) -> RpcResult<PriceResponseBody> {
        let price = self
            .get_token_info(chain_id, address, currency, metrics.clone())
            .await?;
        let response = PriceResponseBody {
            fungibles: vec![FungiblePriceItem {
                address: format!(
                    "{}:{}:{}",
                    crypto::CaipNamespaces::Eip155,
                    chain_id,
                    address
                ),
                name: price.name,
                symbol: price.symbol,
                icon_url: price.logo_uri.unwrap_or_default(),
                price: price
                    .price_usd
                    .unwrap_or("0.0".to_string())
                    .parse()
                    .unwrap_or(0.0),
                decimals: price.decimals,
            }],
        };

        Ok(response)
    }
}
