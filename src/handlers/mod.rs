use {
    serde::{Deserialize, Serialize},
    wc::metrics::TaskMetrics,
};

pub mod health;
pub mod history;
pub mod identity;
pub mod metrics;
pub mod proxy;
pub mod ws_proxy;

static HANDLER_TASK_METRICS: TaskMetrics = TaskMetrics::new("handler_task");

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcQueryParams {
    pub chain_id: String,
    pub project_id: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    status: String,
}

// TODO: https://developers.zerion.io/reference/listwallettransactions
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQueryParams {
    pub currency: Option<String>,
    pub project_id: String,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryResponseBody {
    pub data: Vec<HistoryTransaction>,
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransaction {
    pub id: String,
    pub metadata: HistoryTransactionMetadata,
    pub transfers: Vec<HistoryTransactionTransferInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransactionMetadata {
    pub operation_type: String,
    pub hash: String,
    pub mined_at: String,
    pub sent_from: String,
    pub sent_to: String,
    pub status: String,
    pub nonce: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransactionFungibleInfo {
    pub name: String,
    pub symbol: String,
    pub icon_url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransactionTransferInfo {
    pub fungible_info: Option<HistoryTransactionFungibleInfo>,
    pub nft_info: Option<HistoryTransactionNFTInfo>,
    pub direction: String,
    pub value: usize,
    pub price: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransactionNFTInfo {
    pub name: String,
    pub content: HistoryTransactionNFTContent,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransactionNFTContent {
    pub preview: HistoryTransactionNFTContentItem,
    pub detail: HistoryTransactionNFTContentItem,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransactionNFTContentItem {
    pub url: String,
    pub content_type: String,
}
