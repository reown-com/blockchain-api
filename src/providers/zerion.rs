use {
    super::{HistoryProvider, PortfolioProvider},
    crate::{
        error::{RpcError, RpcResult},
        handlers::{
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
        utils::crypto,
    },
    async_trait::async_trait,
    axum::body::Bytes,
    futures_util::StreamExt,
    hyper::Client,
    hyper_tls::HttpsConnector,
    serde::{Deserialize, Serialize},
    tracing::log::{error, info},
    url::Url,
};

#[derive(Debug)]
pub struct ZerionProvider {
    pub api_key: String,
    pub http_client: Client<HttpsConnector<hyper::client::HttpConnector>>,
}

impl ZerionProvider {
    pub fn new(api_key: String) -> Self {
        let http_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        Self {
            api_key,
            http_client,
        }
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
    pub quantity: ZerionPortfolioQuantity,
    pub fungible_info: ZerionPortfolioFungibleInfo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionPortfolioQuantity {
    pub numeric: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionPortfolioFungibleInfo {
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionTransactionsReponseBody {
    pub r#type: String,
    pub id: String,
    pub attributes: ZerionTransactionAttributes,
    pub relationships: ZerionTransactionsRelationships,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionTransactionsRelationships {
    pub chain: ZerionTransactionsRelationshipsChain,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionTransactionsRelationshipsChain {
    pub data: ZerionTransactionsRelationshipsChainData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ZerionTransactionsRelationshipsChainData {
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
    pub fungible_info: Option<ZerionTransactionFungibleInfo>,
    pub nft_info: Option<ZerionTransactionNFTInfo>,
    pub direction: String,
    pub quantity: ZerionTransactionTransferQuantity,
    pub value: Option<f64>,
    pub price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ZerionTransactionFungibleInfo {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub icon: Option<ZerionTransactionURLItem>,
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

#[async_trait]
impl HistoryProvider for ZerionProvider {
    #[tracing::instrument(skip(self, body, params), fields(provider = "Zerion"))]
    async fn get_transactions(
        &self,
        address: String,
        body: Bytes,
        params: HistoryQueryParams,
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

        let hyper_request = hyper::http::Request::builder()
            .uri(url.as_str())
            .header("Content-Type", "application/json")
            .header("authorization", format!("Basic {}", self.api_key))
            .body(hyper::body::Body::from(body))?;

        let response = self.http_client.request(hyper_request).await?;

        if !response.status().is_success() {
            error!(
                "Error on zerion transactions response. Status is not OK: {:?}",
                response.status()
            );
            return Err(RpcError::TransactionProviderError);
        }

        let mut body = response.into_body();
        let mut bytes = Vec::new();
        while let Some(next) = body.next().await {
            bytes.extend_from_slice(&next?);
        }
        let body: ZerionResponseBody<Vec<ZerionTransactionsReponseBody>> =
            match serde_json::from_slice(&bytes) {
                Ok(body) => body,
                Err(e) => {
                    error!("Error on parsing zerion transactions response: {:?}", e);
                    return Err(RpcError::TransactionProviderError);
                }
            };

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
                        // Try to convert chain id from human readable to caip2 format
                        match crypto::string_chain_id_to_caip2_format(
                            &f.relationships.chain.data.id,
                        ) {
                            Ok(chain) => Some(chain),
                            Err(_) => {
                                info!(
                                    "Error on parsing chain id to caip2 format: {:?}",
                                    f.relationships.chain.data.id
                                );
                                None
                            }
                        }
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
                                    symbol: f.symbol,
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

        let hyper_request = hyper::http::Request::builder()
            .uri(url.as_str())
            .header("Content-Type", "application/json")
            .header("authorization", format!("Basic {}", self.api_key))
            .body(hyper::body::Body::from(body))?;

        let response = self.http_client.request(hyper_request).await?;

        if !response.status().is_success() {
            error!(
                "Error on zerion portfolio response. Status is not OK: {:?}",
                response.status()
            );
            return Err(RpcError::PortfolioProviderError);
        }

        let mut body = response.into_body();
        let mut bytes = Vec::new();
        while let Some(next) = body.next().await {
            bytes.extend_from_slice(&next?);
        }
        let body: ZerionResponseBody<Vec<ZerionPortfolioResponseBody>> =
            match serde_json::from_slice(&bytes) {
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
