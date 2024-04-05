use {
    super::{BalanceProvider, FungiblePriceProvider, HistoryProvider, PortfolioProvider},
    crate::{
        error::{RpcError, RpcResult},
        handlers::{
            balance::{BalanceQueryParams, BalanceResponseBody},
            fungible_price::{FungiblePriceItem, PriceCurrencies, PriceResponseBody},
            history::{
                HistoryQueryParams, HistoryResponseBody, HistoryTransaction,
                HistoryTransactionFungibleInfo, HistoryTransactionMetadata,
                HistoryTransactionMetadataApplication, HistoryTransactionNFTContent,
                HistoryTransactionNFTInfo, HistoryTransactionNFTInfoFlags,
                HistoryTransactionTransfer, HistoryTransactionTransferQuantity,
                HistoryTransactionURLItem, HistoryTransactionURLandContentTypeItem,
            },
            portfolio::{PortfolioPosition, PortfolioQueryParams, PortfolioResponseBody},
        },
        providers::balance::{BalanceItem, BalanceQuantity},
        utils::crypto,
    },
    async_trait::async_trait,
    axum::body::Bytes,
    serde::{Deserialize, Serialize},
    tracing::log::error,
    url::Url,
};

#[derive(Debug)]
pub struct ZerionProvider {
    pub api_key: String,
}

impl ZerionProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
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
    pub name: String,
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
    pub price: f64,
}

#[async_trait]
impl HistoryProvider for ZerionProvider {
    #[tracing::instrument(skip(self, params), fields(provider = "Zerion"))]
    async fn get_transactions(
        &self,
        address: String,
        params: HistoryQueryParams,
        http_client: reqwest::Client,
    ) -> RpcResult<HistoryResponseBody> {
        let base = format!(
            "https://api.zerion.io/v1/wallets/{}/transactions/?",
            &address
        );
        let mut url = Url::parse(&base).map_err(|_| RpcError::HistoryParseCursorError)?;
        url.query_pairs_mut()
            .append_pair("currency", &params.currency.unwrap_or("usd".to_string()));

        if let Some(cursor) = params.cursor {
            url.query_pairs_mut().append_pair("page[after]", &cursor);
        }

        let response = http_client
            .get(url)
            .header("Content-Type", "application/json")
            .header("authorization", format!("Basic {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            error!(
                "Error on zerion transactions response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::TransactionProviderError);
        }
        let body = response
            .json::<ZerionResponseBody<Vec<ZerionTransactionsReponseBody>>>()
            .await?;

        let next: Option<String> = match body.links.next {
            Some(url) => {
                let url = Url::parse(&url).map_err(|_| RpcError::HistoryParseCursorError)?;
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
                                    name: Some(f.name),
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
}

#[async_trait]
impl PortfolioProvider for ZerionProvider {
    #[tracing::instrument(skip(self, body, params), fields(provider = "Zerion"))]
    async fn get_portfolio(
        &self,
        address: String,
        body: Bytes,
        params: PortfolioQueryParams,
    ) -> RpcResult<PortfolioResponseBody> {
        let base = format!("https://api.zerion.io/v1/wallets/{}/positions/?", &address);
        let mut url = Url::parse(&base).map_err(|_| RpcError::HistoryParseCursorError)?;
        url.query_pairs_mut()
            .append_pair("currency", &params.currency.unwrap_or("usd".to_string()));

        let response = reqwest::Client::new()
            .get(url.as_str())
            .header("Content-Type", "application/json")
            .header("authorization", format!("Basic {}", self.api_key))
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            error!(
                "Error on zerion portfolio response. Status is not OK: {:?}",
                response.status()
            );
            return Err(RpcError::PortfolioProviderError);
        }

        let body = response.bytes().await?;
        let body: ZerionResponseBody<Vec<ZerionPortfolioResponseBody>> =
            match serde_json::from_slice(&body) {
                Ok(body) => body,
                Err(e) => {
                    error!("Error on parsing zerion portfolio response: {:?}", e);
                    return Err(RpcError::PortfolioProviderError);
                }
            };

        let portfolio = body
            .data
            .into_iter()
            .map(|f| PortfolioPosition {
                id: f.id,
                name: f.attributes.fungible_info.name,
                symbol: f.attributes.fungible_info.symbol,
            })
            .collect();

        Ok(PortfolioResponseBody { data: portfolio })
    }
}

#[async_trait]
impl BalanceProvider for ZerionProvider {
    #[tracing::instrument(skip(self, params), fields(provider = "Zerion"))]
    async fn get_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        http_client: reqwest::Client,
    ) -> RpcResult<BalanceResponseBody> {
        let base = format!("https://api.zerion.io/v1/wallets/{}/positions/?", &address);
        let mut url = Url::parse(&base).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut()
            .append_pair("currency", &params.currency.to_string());
        url.query_pairs_mut()
            .append_pair("filter[position_types]", "wallet");

        if let Some(chain_id) = params.chain_id {
            let chain_name = crypto::ChainId::from_caip2(&chain_id)
                .ok_or(RpcError::InvalidParameter(chain_id))?;
            url.query_pairs_mut()
                .append_pair("filter[chain_ids]", &chain_name);
        }

        let response = http_client
            .get(url)
            .header("Content-Type", "application/json")
            .header("authorization", format!("Basic {}", self.api_key))
            .send()
            .await?;

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

        const POLYGON_NATIVE_TOKEN_ADDRESS: &str = "0x0000000000000000000000000000000000001010";

        let balances_vec = body
            .data
            .into_iter()
            .map(|f| BalanceItem {
                name: f.attributes.fungible_info.name,
                symbol: f.attributes.fungible_info.symbol,
                chain_id: crypto::ChainId::to_caip2(&f.relationships.chain.data.id),
                address: {
                    let chain_id_human = f.relationships.chain.data.id;
                    let chain_address = f
                        .attributes
                        .fungible_info
                        .implementations
                        .iter()
                        .find(|f| f.chain_id == chain_id_human)
                        .and_then(|f| f.address.clone());
                    let chain_id = crypto::ChainId::to_caip2(&chain_id_human);
                    if let Some(chain_address) = chain_address {
                        // For Polygon native token (Matic)
                        // address is returned, but address should be null
                        // for native tokens
                        // https://specs.walletconnect.com/2.0/specs/servers/blockchain/blockchain-server-api#success-response-body-4
                        if chain_address == POLYGON_NATIVE_TOKEN_ADDRESS {
                            None
                        } else {
                            chain_id.map(|chain_id| format!("{}:{}", &chain_id, chain_address))
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
                icon_url: f
                    .attributes
                    .fungible_info
                    .icon
                    .map(|f| f.url)
                    .unwrap_or_default(),
            })
            .collect();

        let response = BalanceResponseBody {
            balances: balances_vec,
        };

        Ok(response)
    }
}

#[async_trait]
impl FungiblePriceProvider for ZerionProvider {
    #[tracing::instrument(skip(self, http_client), fields(provider = "Zerion"))]
    async fn get_price(
        &self,
        chain_id: &str,
        address: &str,
        currency: &PriceCurrencies,
        http_client: reqwest::Client,
    ) -> RpcResult<PriceResponseBody> {
        let base = "https://api.zerion.io/v1/fungibles/?".to_string();
        let mut url = Url::parse(&base).map_err(|_| RpcError::FungiblePriceParseURLError)?;
        let currency = format!("{}", currency);

        url.query_pairs_mut().append_pair("currency", &currency);
        url.query_pairs_mut()
            .append_pair("filter[chain_id]", chain_id);
        url.query_pairs_mut()
            .append_pair("filter[implementation_address]", address);

        let response = http_client
            .get(url)
            .header("Content-Type", "application/json")
            .header("authorization", format!("Basic {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            error!(
                "Error on zerion fungibles price response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::FungiblePriceProviderError);
        }
        let body = response
            .json::<ZerionResponseBody<Vec<ZerionFungibleAsset>>>()
            .await?;

        let fungibles_vec: Vec<FungiblePriceItem> = body
            .data
            .into_iter()
            .map(|f| FungiblePriceItem {
                name: f.attributes.name,
                symbol: f.attributes.symbol,
                icon_url: f.attributes.icon.map(|f| f.url).unwrap_or_default(),
                price: f.attributes.market_data.price,
            })
            .collect();

        let response = PriceResponseBody {
            fungibles: fungibles_vec,
        };

        Ok(response)
    }
}
