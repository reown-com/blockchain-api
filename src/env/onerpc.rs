use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct OneRpcConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for OneRpcConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for OneRpcConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::OneRpc
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum mainnet
        (
            "eip155:1".into(),
            ("eth".into(), Weight::new(Priority::Minimal).unwrap()),
        ),
        // Arbitrum One
        (
            "eip155:42161".into(),
            ("arb".into(), Weight::new(Priority::Low).unwrap()),
        ),
        // BSC
        (
            "eip155:56".into(),
            ("bnb".into(), Weight::new(Priority::Low).unwrap()),
        ),
        // Polygon
        (
            "eip155:137".into(),
            ("matic".into(), Weight::new(Priority::Low).unwrap()),
        ),
        // Base
        (
            "eip155:8453".into(),
            ("base".into(), Weight::new(Priority::Low).unwrap()),
        ),
        // Klaytn
        (
            "eip155:8217".into(),
            ("klay".into(), Weight::new(Priority::Low).unwrap()),
        ),
    ])
}
