use url::Url;

#[cfg(feature = "test-localhost")]
use {rpc_proxy::test_helpers::spawn_blockchain_api, std::env};

pub struct RpcProxy {
    pub public_addr: Url,
    pub project_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[cfg(feature = "test-localhost")]
impl RpcProxy {
    pub async fn start() -> Self {
        let public_addr = spawn_blockchain_api().await;

        let project_id =
            env::var("TEST_RPC_PROXY_PROJECT_ID").expect("TEST_RPC_PROXY_PROJECT_ID must be set");

        Self {
            public_addr,
            project_id,
        }
    }
}
