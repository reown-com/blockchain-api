use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ZKSyncConfig {
    pub project_id: String,

    #[serde(default = "default_zksync_supported_chains")]
    pub supported_chains: HashMap<String, String>,
}

impl Default for ZKSyncConfig {
    fn default() -> Self {
        Self {
            project_id: Default::default(),
            supported_chains: default_zksync_supported_chains(),
        }
    }
}

fn default_zksync_supported_chains() -> HashMap<String, String> {
    HashMap::from([
        // zkSync Testnet
        (
            "eip155:280".into(),
            "https://zksync2-testnet.zksync.dev".into(),
        ),
        // zkSync Mainnet
        (
            "eip155:324".into(),
            "https://zksync2-mainnet.zksync.io".into(),
        ),
    ])
}
