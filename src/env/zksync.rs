use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct ZKSyncConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for ZKSyncConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for ZKSyncConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::ZKSync
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // zkSync Sepolia Testnet
        (
            "eip155:300".into(),
            (
                "https://sepolia.era.zksync.dev".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // zkSync Mainnet
        (
            "eip155:324".into(),
            (
                "https://mainnet.era.zksync.io".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
