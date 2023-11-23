use {
    super::HistoryProvider,
    crate::{
        error::{RpcError, RpcResult},
        handlers::{
            HistoryQueryParams,
            HistoryResponseBody,
            HistoryTransaction,
            HistoryTransactionMetadata,
        },
    },
    async_trait::async_trait,
    axum::{body::Bytes, http::method},
    futures_util::StreamExt,
    hyper::Client,
    hyper_tls::HttpsConnector,
    serde::{Deserialize, Serialize},
    tracing::log::error,
    url::Url,
};

#[derive(Debug)]
pub struct CoinbaseProvider {
    pub api_key: String,
    pub app_id: String,
    pub http_client: Client<HttpsConnector<hyper::client::HttpConnector>>,
}

impl CoinbaseProvider {
    pub fn new(api_key: String, app_id: String) -> Self {
        let http_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        Self {
            api_key,
            app_id,
            http_client,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CoinbaseResponseBody {
    pub transactions: Vec<CoinbaseTransaction>,
    pub next_page_key: Option<String>,
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CoinbaseTransaction {
    pub status: String,
    pub transaction_id: String,
    pub tx_hash: String,
    pub created_at: String,
}

#[async_trait]
impl HistoryProvider for CoinbaseProvider {
    async fn get_transactions(
        &self,
        address: String,
        body: Bytes,
        params: HistoryQueryParams,
    ) -> RpcResult<HistoryResponseBody> {
        let base = format!(
            "https://pay.coinbase.com/api/v1/buy/{}/transactions",
            &address
        );
        let mut url = Url::parse(&base).map_err(|_| RpcError::HistoryParseCursorError)?;
        url.query_pairs_mut().append_pair("page_size", "50");

        if let Some(cursor) = params.cursor {
            url.query_pairs_mut().append_pair("page_key", &cursor);
        }

        error!(
            "Coinbase URL: {:?}, {:?}",
            url.as_str(),
            self.app_id.clone()
        );

        let hyper_request = hyper::http::Request::builder()
            .uri(url.as_str())
            .method(method::Method::GET)
            .header("Content-Type", "application/json")
            .header("CBPAY-APP-ID", self.app_id.clone())
            .header("CBPAY-API-KEY", self.api_key.clone())
            .body(hyper::body::Body::from(body))?;

        let response = self.http_client.request(hyper_request).await?;

        if !response.status().is_success() {
            error!(
                "Error on Coinbase transactions response. Status is not OK: {:?}",
                response.status()
            );
            return Err(RpcError::TransactionProviderError);
        }

        let mut body = response.into_body();
        let mut bytes = Vec::new();
        while let Some(next) = body.next().await {
            bytes.extend_from_slice(&next?);
        }
        let body: CoinbaseResponseBody = match serde_json::from_slice(&bytes) {
            Ok(body) => body,
            Err(e) => {
                error!("Error on parsing coinbase transactions response: {:?}", e);
                return Err(RpcError::TransactionProviderError);
            }
        };

        let transactions = body
            .transactions
            .into_iter()
            .map(|f| HistoryTransaction {
                id: f.transaction_id,
                metadata: HistoryTransactionMetadata {
                    operation_type: "buy".to_string(),
                    hash: f.tx_hash,
                    mined_at: f.created_at,
                    nonce: 1, // TODO: get nonce from somewhere
                    sent_from: "Coinbase".to_string(),
                    sent_to: address.clone(),
                    status: f.status,
                },
                transfers: None,
            })
            .collect();

        Ok(HistoryResponseBody {
            data: transactions,
            next: body.next_page_key,
        })
    }
}
