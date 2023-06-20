use serde::{Deserialize, Serialize};

pub mod health;
pub mod identity;
pub mod metrics;
pub mod proxy;
pub mod ws_proxy;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcQueryParams {
    pub chain_id: String,
    pub project_id: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    status: String,
}
