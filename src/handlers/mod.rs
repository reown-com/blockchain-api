use {
    serde::{Deserialize, Serialize},
    wc::metrics::TaskMetrics,
};

pub mod balance;
pub mod convert;
pub mod generators;
pub mod health;
pub mod history;
pub mod identity;
pub mod metrics;
pub mod onramp;
pub mod portfolio;
pub mod profile;
pub mod proxy;
pub mod ws_proxy;
pub mod supported_chains;

static HANDLER_TASK_METRICS: TaskMetrics = TaskMetrics::new("handler_task");

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcQueryParams {
    pub chain_id: String,
    pub project_id: String,
    /// Optional provider ID for the exact provider request
    pub provider_id: Option<String>,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    status: String,
}
