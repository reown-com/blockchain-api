use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct AllnodesConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
    pub supported_ws_chains: HashMap<String, (String, Weight)>,
    pub api_key: String,
}

impl AllnodesConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            supported_chains: default_supported_chains(),
            supported_ws_chains: default_ws_supported_chains(),
            api_key,
        }
    }
}

impl ProviderConfig for AllnodesConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_ws_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Allnodes
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md and src/chain_config.ACTIVE_CONFIG

    HashMap::from([
        // Ethereum Mainnet
        (
            "eip155:1".into(),
            ("eth57873".into(), Weight::new(Priority::Max).unwrap()),
        ),
    ])
}

fn default_ws_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md and src/chain_config.ACTIVE_CONFIG

    HashMap::from([
        // Ethereum
        (
            "eip155:1".into(),
            ("eth57873".into(), Weight::new(Priority::Normal).unwrap()),
        ),
    ])
}
