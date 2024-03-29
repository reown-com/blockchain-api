use {
    crate::{state::AppState, utils::network},
    axum::extract::{ConnectInfo, MatchedPath, State},
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, sync::Arc},
    wc::{metrics::TaskMetrics, rate_limit::RateLimitExceeded},
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
pub mod supported_chains;
pub mod ws_proxy;

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

/// Checking rate limit for the request in the handler
pub async fn handle_rate_limit(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    connect_info: ConnectInfo<SocketAddr>,
    path: MatchedPath,
    project_id: Option<&str>,
) -> Result<(), RateLimitExceeded> {
    state
        .rate_limit
        .as_ref()
        .unwrap()
        .is_rate_limited(
            path.as_str(),
            &network::get_forwarded_ip(headers.clone())
                .unwrap_or_else(|| connect_info.0.ip())
                .to_string(),
            project_id,
        )
        .await
}
