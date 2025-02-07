use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct AllnodesConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
    pub api_key: String,
}

impl AllnodesConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            supported_chains: default_supported_chains(),
            api_key,
        }
    }
}

impl ProviderConfig for AllnodesConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Allnodes
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum Mainnet
        (
            "eip155:1".into(),
            ("eth57873".into(), Weight::new(Priority::Max).unwrap()),
        ),
    ])
}
