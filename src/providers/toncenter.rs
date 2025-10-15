use {
    super::{HistoryProvider, ProviderKind, TokenMetadataCacheProvider},
    crate::{
        error::{RpcError, RpcResult},
        handlers::history::{
            HistoryQueryParams, HistoryResponseBody, HistoryTransaction,
            HistoryTransactionFungibleInfo, HistoryTransactionMetadata, HistoryTransactionTransfer,
            HistoryTransactionTransferQuantity, HistoryTransactionURLItem,
        },
        utils::crypto,
        Metrics,
    },
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    tap::TapFallible,
    tracing::error,
    url::Url,
};

const TON_MAINNET_CHAIN_ID: &str = "ton:mainnet";
const TON_NATIVE_TOKEN_SYMBOL: &str = "TON";
const TON_NATIVE_TOKEN_NAME: &str = "Toncoin";
const TON_NATIVE_TOKEN_ICON: &str = "https://ton.org/img/ton_symbol.png";
const TONCENTER_HISTORY_PAGE_SIZE: u32 = 100;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TonV3TransactionsResponse {
    pub transactions: Vec<TonTransaction>,
    #[serde(default)]
    pub address_book: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TonTransaction {
    #[serde(alias = "now", alias = "utime")]
    pub utime: u64,
    #[serde(default)]
    pub lt: Option<String>,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub transaction_id: Option<TonTxId>,
    pub data: Option<String>,
    pub in_msg: Option<TonMessage>,
    pub out_msgs: Option<Vec<TonMessage>>,
    pub fee: Option<String>,
    pub storage_fee: Option<String>,
    pub other_fee: Option<String>,
    #[serde(default)]
    pub prev_trans_lt: Option<String>,
    #[serde(default)]
    pub prev_trans_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TonTxId {
    pub lt: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TonMessage {
    pub source: Option<String>,
    pub destination: Option<String>,
    pub value: Option<String>, // in nanotons for native transfers
    pub msg_data: Option<serde_json::Value>,
}

pub struct ToncenterProvider {
    provider_kind: ProviderKind,
    api_url: String,
    api_key: Option<String>,
    http_client: reqwest::Client,
}

impl ToncenterProvider {
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        Self {
            provider_kind: ProviderKind::Toncenter,
            api_url,
            api_key,
            http_client: reqwest::Client::new(),
        }
    }

    async fn send_request(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        let mut req = self.http_client.get(url.clone());
        if let Some(key) = &self.api_key {
            req = req.header("X-Api-Key", key);
        }
        let response = req.send().await;
        response
    }

    fn build_history_url(
        &self,
        address: &str,
        limit: u32,
        before_lt: Option<String>,
        before_hash: Option<String>,
    ) -> Result<Url, RpcError> {
        let base = format!("{}/api/v3/transactions", self.api_url.trim_end_matches('/'));
        let mut url = Url::parse(&base).map_err(|_| RpcError::HistoryParseCursorError)?;
        url.query_pairs_mut().append_pair("account", address);
        url.query_pairs_mut()
            .append_pair("limit", &limit.to_string());
        if let Some(lt) = before_lt {
            url.query_pairs_mut().append_pair("before_lt", &lt);
        }
        if let Some(hash) = before_hash {
            url.query_pairs_mut().append_pair("before_hash", &hash);
        }
        Ok(url)
    }
}

#[async_trait]
impl HistoryProvider for ToncenterProvider {
    async fn get_transactions(
        &self,
        address: String,
        params: HistoryQueryParams,
        _metadata_cache: &Arc<dyn TokenMetadataCacheProvider>,
        metrics: Arc<Metrics>,
    ) -> RpcResult<HistoryResponseBody> {
        let (mut before_lt, mut before_hash) = (None, None);
        if let Some(cursor) = params.cursor.clone() {
            let parts: Vec<&str> = cursor.split(':').collect();
            if parts.len() == 2 {
                before_lt = Some(parts[0].to_string());
                before_hash = Some(parts[1].to_string());
            }
        }
        let url = self.build_history_url(
            &address,
            TONCENTER_HISTORY_PAGE_SIZE,
            before_lt,
            before_hash,
        )?;

        let latency_start = std::time::SystemTime::now();
        let response = self.send_request(url).await.tap_err(|e| {
            error!("Error on Toncenter history request with {e}");
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("v3/transactions".to_string()),
        );
        if !response.status().is_success() {
            error!(
                "Error on Toncenter history response. Status is not OK: {:?}",
                response
            );
            return Err(RpcError::TransactionProviderError);
        }

        let v3: TonV3TransactionsResponse = response.json().await.map_err(|e| {
            error!("Error on Toncenter history response with {e}");
            RpcError::TransactionProviderError
        })?;
        let transactions: Vec<TonTransaction> = v3.transactions;

        let mut history: Vec<HistoryTransaction> = Vec::new();
        for tx in transactions.iter() {
            let from_raw = tx
                .in_msg
                .as_ref()
                .and_then(|m| m.source.clone())
                .unwrap_or_default();
            let to_raw = tx
                .in_msg
                .as_ref()
                .and_then(|m| m.destination.clone())
                .unwrap_or_default();
            let from = crypto::to_friendly_if_raw(&from_raw);
            let to = crypto::to_friendly_if_raw(&to_raw);

            // Resolve lt/hash either from top-level or from transaction_id
            let (lt, hash) = match (&tx.lt, &tx.hash, &tx.transaction_id) {
                (Some(lt), Some(hash), _) => (lt.clone(), hash.clone()),
                (_, _, Some(id)) => (id.lt.clone(), id.hash.clone()),
                _ => (String::new(), String::new()),
            };
            let mined_at = chrono::DateTime::from_timestamp(tx.utime as i64, 0)
                .unwrap_or(chrono::DateTime::from_timestamp(0, 0).unwrap())
                .to_utc()
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string();

            let transfer_opt = tx
                .in_msg
                .as_ref()
                .and_then(|m| m.value.as_ref())
                .and_then(|v| v.parse::<u128>().ok())
                .map(|nanotons| {
                    let amount = (nanotons as f64) / 1_000_000_000f64;
                    HistoryTransactionTransfer {
                        fungible_info: Some(HistoryTransactionFungibleInfo {
                            name: Some(TON_NATIVE_TOKEN_NAME.to_string()),
                            symbol: Some(TON_NATIVE_TOKEN_SYMBOL.to_string()),
                            icon: Some(HistoryTransactionURLItem {
                                url: TON_NATIVE_TOKEN_ICON.to_string(),
                            }),
                        }),
                        nft_info: None,
                        direction: if to.eq_ignore_ascii_case(&address) {
                            "in".to_string()
                        } else {
                            "out".to_string()
                        },
                        quantity: HistoryTransactionTransferQuantity {
                            numeric: amount.to_string(),
                        },
                        value: None,
                        price: None,
                    }
                });

            let tx_item = HistoryTransaction {
                id: lt.clone(),
                metadata: HistoryTransactionMetadata {
                    operation_type: match &transfer_opt {
                        Some(t) => {
                            if t.direction == "in" {
                                "receive".to_string()
                            } else {
                                "send".to_string()
                            }
                        }
                        None => "execute".to_string(),
                    },
                    hash,
                    mined_at,
                    sent_from: from,
                    sent_to: to,
                    status: "confirmed".to_string(),
                    nonce: 0,
                    application: None,
                    chain: Some(TON_MAINNET_CHAIN_ID.to_string()),
                },
                transfers: transfer_opt.map(|t| vec![t]),
            };
            history.push(tx_item);
        }

        // next cursor: use last tx's prev_trans_* if available, otherwise last tx's lt/hash
        let next = if let Some(last) = transactions.last() {
            match (&last.prev_trans_lt, &last.prev_trans_hash) {
                (Some(plt), Some(ph)) if !plt.is_empty() && !ph.is_empty() => {
                    Some(format!("{}:{}", plt, ph))
                }
                _ => {
                    let (lt, hash) = match (&last.lt, &last.hash, &last.transaction_id) {
                        (Some(lt), Some(hash), _) => (lt.clone(), hash.clone()),
                        (_, _, Some(id)) => (id.lt.clone(), id.hash.clone()),
                        _ => (String::new(), String::new()),
                    };
                    if lt.is_empty() || hash.is_empty() {
                        None
                    } else {
                        Some(format!("{lt}:{hash}"))
                    }
                }
            }
        } else {
            None
        };

        Ok(HistoryResponseBody {
            data: history,
            next,
        })
    }

    fn provider_kind(&self) -> ProviderKind {
        self.provider_kind.clone()
    }
}
