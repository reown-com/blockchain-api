use {
    crate::providers::zerion::ZerionTransactionTransfer,
    serde::{Deserialize, Serialize},
    wc::metrics::TaskMetrics,
};

pub mod health;
pub mod history;
pub mod identity;
pub mod metrics;
pub mod portfolio;
pub mod profile;
pub mod proxy;
pub mod ws_proxy;

static HANDLER_TASK_METRICS: TaskMetrics = TaskMetrics::new("handler_task");

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcQueryParams {
    pub chain_id: String,
    pub project_id: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    status: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioQueryParams {
    pub project_id: String,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioResponseBody {
    pub data: Vec<PortfolioPosition>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioPosition {
    pub id: String,
    pub name: String,
    pub symbol: String,
}

// TODO: https://developers.zerion.io/reference/listwallettransactions
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQueryParams {
    pub currency: Option<String>,
    pub project_id: String,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryResponseBody {
    pub data: Vec<HistoryTransaction>,
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransaction {
    pub id: String,
    pub metadata: HistoryTransactionMetadata,
    pub transfers: Vec<ZerionTransactionTransfer>,
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
