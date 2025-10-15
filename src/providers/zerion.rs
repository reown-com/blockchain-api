use {
    super::{BalanceProvider, BalanceProviderFactory, HistoryProvider, PortfolioProvider},
    crate::{
        env::ZerionConfig,
        error::{RpcError, RpcResult},
        handlers::{
            balance::{
                BalanceQueryParams,
                BalanceResponseBody,
                TokenMetadataCacheItem,
                H160_EMPTY_ADDRESS,
            },
            history::{
                HistoryQueryParams,
                HistoryResponseBody,
                HistoryTransaction,
                HistoryTransactionFungibleInfo,
                HistoryTransactionMetadata,
                HistoryTransactionMetadataApplication,
                HistoryTransactionNFTContent,
                HistoryTransactionNFTInfo,
                HistoryTransactionNFTInfoFlags,
                HistoryTransactionTransfer,
                HistoryTransactionTransferQuantity,
                HistoryTransactionURLItem,
                HistoryTransactionURLandContentTypeItem,
            },
            portfolio::{PortfolioPosition, PortfolioQueryParams, PortfolioResponseBody},
        },
        providers::{
            balance::{BalanceItem, BalanceQuantity},
            ProviderKind,
            TokenMetadataCacheProvider,
        },
        utils::crypto,
        Metrics,
    },
    async_trait::async_trait,
    deadpool_redis::Pool,
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    tap::TapFallible,
    tracing::log::error,
    url::Url,
};

const POLYGON_NATIVE_TOKEN_ADDRESS: &str = "0x0000000000000000000000000000000000001010";

#[derive(Debug)]
pub struct ZerionProvider {
    pub provider_kind: ProviderKind,
    pub api_key: String,
    pub http_client: reqwest::Client,
}

impl ZerionProvider {
    pub fn new(api_key: String) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            provider_kind: ProviderKind::Zerion,
            api_key,
            http_client,
        }
    }

    async fn send_request(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("authorization", format!("Basic {}", self.api_key))
            .send()
            .await
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionResponseBody<T> {
    pub links: ZerionResponseLinks,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionResponseLinks {
    #[serde(rename = "self")]
    pub self_id: String,
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionPortfolioResponseBody {
    pub r#type: String,
    pub id: String,
    pub attributes: ZerionPortfolioAttributes,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionPortfolioAttributes {
    pub quantity: ZerionQuantityAttribute,
    pub fungible_info: ZerionFungibleInfoAttribute,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionQuantityAttribute {
    pub decimals: usize,
    pub numeric: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionTransactionsReponseBody {
    pub r#type: String,
    pub id: String,
    pub attributes: ZerionTransactionAttributes,
    pub relationships: ZerionRelationshipsItem,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionRelationshipsItem {
    pub chain: ZerionRelationshipsItemChain,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionRelationshipsItemChain {
    pub data: ZerionRelationshipsItemChainData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionRelationshipsItemChainData {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionTransactionAttributes {
    pub operation_type: String,
    pub hash: String,
    pub mined_at_block: usize,
    pub mined_at: String,
    pub sent_from: String,
    pub sent_to: String,
    pub status: String,
    pub nonce: usize,
    pub transfers: Vec<ZerionTransactionTransfer>,
    pub application_metadata: Option<ZerionTransactionApplicationMetadata>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionTransactionTransfer {
    pub fungible_info: Option<ZerionFungibleInfoAttribute>,
    pub nft_info: Option<ZerionTransactionNFTInfo>,
    pub direction: String,
    pub quantity: ZerionTransactionTransferQuantity,
    pub value: Option<f64>,
    pub price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionFungibleInfoAttribute {
    pub name: Option<String>,
    pub symbol: String,
    pub icon: Option<ZerionTransactionURLItem>,
    pub implementations: Vec<ZerionImplementation>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionTransactionURLItem {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionTransactionTransferQuantity {
    pub numeric: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionTransactionNFTInfo {
    pub name: Option<String>,
    pub content: Option<ZerionTransactionNFTContent>,
    pub flags: ZerionTransactionNFTInfoFlags,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionTransactionNFTInfoFlags {
    pub is_spam: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionTransactionNFTContent {
    pub preview: Option<ZerionTransactionURLandContentTypeItem>,
    pub detail: Option<ZerionTransactionURLandContentTypeItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionTransactionURLandContentTypeItem {
    pub url: String,
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionTransactionApplicationMetadata {
    pub name: Option<String>,
    pub icon: Option<ZerionUrlItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionUrlItem {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionImplementation {
    pub chain_id: String,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionPosition {
    pub attributes: ZerionPositionAttributes,
    pub relationships: ZerionRelationshipsItem,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionPositionAttributes {
    pub value: Option<f64>,
    pub price: f64,
    pub quantity: ZerionQuantityAttribute,
    pub fungible_info: ZerionFungibleInfoAttribute,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionFungibleAsset {
    pub attributes: ZerionFungibleAssetAttribute,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionFungibleAssetAttribute {
    pub name: String,
    pub symbol: String,
    pub icon: Option<ZerionTransactionURLItem>,
    pub market_data: ZerionMarketData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionMarketData {
    pub price: Option<f64>,
}

fn add_filter_non_trash_only(url: &mut Url) {
    url.query_pairs_mut()
        .append_pair("filter[trash]", "only_non_trash");
}

#[async_trait]
impl HistoryProvider for ZerionProvider {
    async fn get_transactions(
        &self,
        address: String,
        params: HistoryQueryParams,
        _metadata_cache: &Arc<dyn TokenMetadataCacheProvider>,
        metrics: Arc<Metrics>,
    ) -> RpcResult<HistoryResponseBody> {
        let base = format!(
            "https://api.zerion.io/v1/wallets/{}/transactions/?",
            &address
        );
        let mut url = Url::parse(&base).map_err(|e| {
            error!("Error on parsing zerion history url with {e}");
            RpcError::HistoryParseCursorError
        })?;
        url.query_pairs_mut()
            .append_pair("currency", &params.currency.unwrap_or("usd".to_string()));
        // Return only non-spam transactions
        add_filter_non_trash_only(&mut url);

        if let Some(cursor) = params.cursor {
            url.query_pairs_mut().append_pair("page[after]", &cursor);
        }

        if let Some(chain_id) = params.chain_id {
            let chain_name = if chain_id.contains(':') {
                crypto::ChainId::from_caip2(&chain_id)
                    .ok_or(RpcError::InvalidParameter(chain_id))?
            } else {
                crypto::ChainId::from_caip2(&format!("eip155:{chain_id}"))
                    .ok_or(RpcError::InvalidParameter(chain_id))?
            };
            url.query_pairs_mut()
                .append_pair("filter[chain_ids]", &chain_name);
        }

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error on request to zerion transactions history endpoint with {e}");
            RpcError::TransactionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("transactions".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on zerion transactions response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::TransactionProviderError);
        }
        let body = response
            .json::<ZerionResponseBody<Vec<ZerionTransactionsReponseBody>>>()
            .await
            .tap_err(|e| {
                error!("Error on parsing zerion history body with {e}");
            })?;

        let next: Option<String> = match body.links.next {
            Some(url) => {
                let url = Url::parse(&url).map_err(|e| {
                    error!("Error on parsing zerion history next url with {e}");
                    RpcError::HistoryParseCursorError
                })?;
                // Get the "after" query parameter
                if let Some(after_param) = url.query_pairs().find(|(key, _)| key == "page[after]") {
                    let after_value = after_param.1;
                    Some(after_value.to_string())
                } else {
                    None
                }
            }
            None => None,
        };

        let transactions = body
            .data
            .into_iter()
            .map(|f| HistoryTransaction {
                id: f.id,
                metadata: HistoryTransactionMetadata {
                    operation_type: f.attributes.operation_type,
                    hash: f.attributes.hash,
                    mined_at: f.attributes.mined_at,
                    nonce: f.attributes.nonce,
                    sent_from: f.attributes.sent_from,
                    sent_to: f.attributes.sent_to,
                    status: f.attributes.status,
                    application: f.attributes.application_metadata.map(|f| {
                        HistoryTransactionMetadataApplication {
                            name: f.name,
                            icon_url: f.icon.map(|f| f.url),
                        }
                    }),
                    chain: if f.relationships.chain.data.r#type != "chains" {
                        None
                    } else {
                        crypto::ChainId::to_caip2(&f.relationships.chain.data.id)
                    },
                },
                transfers: f
                    .attributes
                    .transfers
                    .into_iter()
                    .map(|f| {
                        Some(HistoryTransactionTransfer {
                            fungible_info: f.fungible_info.map(|f| {
                                HistoryTransactionFungibleInfo {
                                    name: f.name,
                                    symbol: Some(f.symbol),
                                    icon: f.icon.map(|f| HistoryTransactionURLItem { url: f.url }),
                                }
                            }),
                            nft_info: f.nft_info.map(|f| HistoryTransactionNFTInfo {
                                name: f.name,
                                content: f.content.map(|f| HistoryTransactionNFTContent {
                                    preview: f.preview.map(|f| {
                                        HistoryTransactionURLandContentTypeItem {
                                            url: f.url,
                                            content_type: f.content_type,
                                        }
                                    }),
                                    detail: f.detail.map(|f| {
                                        HistoryTransactionURLandContentTypeItem {
                                            url: f.url,
                                            content_type: f.content_type,
                                        }
                                    }),
                                }),
                                flags: HistoryTransactionNFTInfoFlags {
                                    is_spam: f.flags.is_spam,
                                },
                            }),
                            direction: f.direction,
                            quantity: HistoryTransactionTransferQuantity {
                                numeric: f.quantity.numeric,
                            },
                            value: f.value,
                            price: f.price,
                        })
                    })
                    .collect(),
            })
            .collect();

        Ok(HistoryResponseBody {
            data: transactions,
            next,
        })
    }

    fn provider_kind(&self) -> ProviderKind {
        self.provider_kind.clone()
    }
}

#[async_trait]
impl PortfolioProvider for ZerionProvider {
    #[tracing::instrument(skip(self, params), fields(provider = "Zerion"), level = "debug")]
    async fn get_portfolio(
        &self,
        address: String,
        params: PortfolioQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<PortfolioResponseBody> {
        let base = format!("https://api.zerion.io/v1/wallets/{}/positions/?", &address);
        let mut url = Url::parse(&base).map_err(|_| RpcError::HistoryParseCursorError)?;
        url.query_pairs_mut()
            .append_pair("currency", &params.currency.unwrap_or("usd".to_string()));

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error on request to zerion portfolio endpoint with {e}");
            RpcError::PortfolioProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("positions".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on zerion portfolio response. Status is not OK: {:?}",
                response.status()
            );
            return Err(RpcError::PortfolioProviderError);
        }

        let body = response
            .json::<ZerionResponseBody<Vec<ZerionPortfolioResponseBody>>>()
            .await?;

        let portfolio = body
            .data
            .into_iter()
            .map(|f| PortfolioPosition {
                id: f.id,
                name: f
                    .attributes
                    .fungible_info
                    .name
                    .unwrap_or(f.attributes.fungible_info.symbol.clone()),
                symbol: f.attributes.fungible_info.symbol,
            })
            .collect();

        Ok(PortfolioResponseBody { data: portfolio })
    }
}

#[async_trait]
impl BalanceProvider for ZerionProvider {
    async fn get_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        metadata_cache: &Arc<dyn TokenMetadataCacheProvider>,
        metrics: Arc<Metrics>,
    ) -> RpcResult<BalanceResponseBody> {
        let base = format!("https://api.zerion.io/v1/wallets/{}/positions/?", &address);
        let mut url = Url::parse(&base).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut()
            .append_pair("currency", &params.currency.to_string());
        url.query_pairs_mut()
            .append_pair("filter[position_types]", "wallet");

        // Return only non-spam transactions
        add_filter_non_trash_only(&mut url);

        if let Some(chain_id) = params.chain_id {
            let chain_name = if chain_id.contains(':') {
                crypto::ChainId::from_caip2(&chain_id)
                    .ok_or(RpcError::InvalidParameter(chain_id))?
            } else {
                crypto::ChainId::from_caip2(&format!("eip155:{chain_id}"))
                    .ok_or(RpcError::InvalidParameter(chain_id))?
            };
            url.query_pairs_mut()
                .append_pair("filter[chain_ids]", &chain_name);
        }

        let latency_start = SystemTime::now();
        let response = self.send_request(url.clone()).await.map_err(|e| {
            error!("Error on request to zerion transactions history endpoint with {e}");
            RpcError::BalanceProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("positions".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on zerion balance response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::BalanceProviderError);
        }
        let body = response
            .json::<ZerionResponseBody<Vec<ZerionPosition>>>()
            .await?;

        let mut balances_vec = Vec::new();
        for f in body.data {
            let chain_id_human = f.relationships.chain.data.id;
            let token_address = f
                .attributes
                .fungible_info
                .implementations
                .iter()
                .find(|impl_| impl_.chain_id == chain_id_human)
                .and_then(|impl_| impl_.address.clone());

            let token_address_strict = token_address
                .clone()
                .unwrap_or_else(|| H160_EMPTY_ADDRESS.to_string());
            let chain_id = crypto::ChainId::to_caip2(&chain_id_human);

            // Set the default metadata from the response
            let mut token_metadata = TokenMetadataCacheItem {
                name: f
                    .attributes
                    .fungible_info
                    .name
                    .unwrap_or(f.attributes.fungible_info.symbol.clone()),
                symbol: f.attributes.fungible_info.symbol.clone(),
                icon_url: f
                    .attributes
                    .fungible_info
                    .icon
                    .map(|icon| icon.url)
                    .unwrap_or_default(),
                decimals: f.attributes.quantity.decimals as u8,
            };

            // Update the token metadata from the cache or update the cache if it's not
            // present
            if let Some(chain_id) = chain_id.clone() {
                let caip10_token_address = format!("{chain_id}:{token_address_strict}");
                match metadata_cache.get_metadata(&caip10_token_address).await {
                    Ok(Some(cached_metadata)) => token_metadata = cached_metadata,
                    Ok(None) => {
                        let metadata_cache = metadata_cache.clone();
                        let token_metadata_clone = token_metadata.clone();
                        tokio::spawn(async move {
                            if let Err(e) = metadata_cache
                                .set_metadata(&caip10_token_address, &token_metadata_clone)
                                .await
                            {
                                error!(
                                    "Error setting metadata in cache for {caip10_token_address}: \
                                     {e}"
                                );
                            }
                        });
                    }
                    Err(e) => error!("Error getting metadata from cache: {e}"),
                }
            }

            let balance_item = BalanceItem {
                name: token_metadata.name,
                symbol: token_metadata.symbol,
                chain_id: chain_id.clone(),
                address: {
                    if let Some(addr) = token_address {
                        // For Polygon native token (POL) we set address to None
                        if addr == POLYGON_NATIVE_TOKEN_ADDRESS {
                            None
                        } else {
                            chain_id
                                .as_ref()
                                .map(|chain_id| format!("{chain_id}:{addr}"))
                        }
                    } else {
                        None
                    }
                },
                value: f.attributes.value,
                price: f.attributes.price,
                quantity: BalanceQuantity {
                    decimals: f.attributes.quantity.decimals.to_string(),
                    numeric: f.attributes.quantity.numeric,
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
        self.provider_kind.clone()
    }
}

impl BalanceProviderFactory<ZerionConfig> for ZerionProvider {
    fn new(provider_config: &ZerionConfig, _cache: Option<Arc<Pool>>) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            provider_kind: ProviderKind::Zerion,
            api_key: provider_config.api_key.clone(),
            http_client,
        }
    }
}
