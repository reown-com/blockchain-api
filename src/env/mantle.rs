use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct MantleConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for MantleConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for MantleConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Mantle
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Mantle mainnet
        (
            "eip155:5000".into(),
            (
                "https://rpc.mantle.xyz".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Mantle testnet
        (
            "eip155:5001".into(),
            (
                "https://rpc.testnet.mantle.xyz".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
    ])
}
