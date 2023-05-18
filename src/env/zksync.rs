use {crate::providers::Weight, serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ZKSyncConfig {
    #[serde(default)]
    pub project_id: String,

    #[serde(default = "default_supported_chains")]
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for ZKSyncConfig {
    fn default() -> Self {
        Self {
            project_id: Default::default(),
            supported_chains: default_supported_chains(),
        }
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // zkSync Testnet
        (
            "eip155:280".into(),
            ("https://zksync2-testnet.zksync.dev".into(), Weight(1.0)),
        ),
        // zkSync Mainnet
        (
            "eip155:324".into(),
            ("https://zksync2-mainnet.zksync.io".into(), Weight(1.0)),
        ),
    ])
}
