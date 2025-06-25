use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct RootstockConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for RootstockConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for RootstockConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Rootstock
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Rootstock Mainnet
        (
            "eip155:30".into(),
            (
                "https://public-node.rsk.co".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Rootstock Testnet
        (
            "eip155:31".into(),
            (
                "https://public-node.testnet.rsk.co".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
