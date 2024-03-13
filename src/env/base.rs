use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct BaseConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for BaseConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Base
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Base Mainnet
        (
            "eip155:8453".into(),
            (
                "https://mainnet.base.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Base Sepolia
        (
            "eip155:84532".into(),
            (
                "https://sepolia.base.org".into(),
                Weight::new(Priority::Low).unwrap(),
            ),
        ),
    ])
}
