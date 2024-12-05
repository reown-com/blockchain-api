use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct ArbitrumConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for ArbitrumConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for ArbitrumConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Arbitrum
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Arbitrum One
        (
            "eip155:42161".into(),
            (
                "https://arb1.arbitrum.io/rpc".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Arbitrum Sepolia
        (
            "eip155:421614".into(),
            (
                "https://sepolia-rollup.arbitrum.io/rpc".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
