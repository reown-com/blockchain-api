use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct AuroraConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for AuroraConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for AuroraConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Aurora
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Aurora Mainnet
        (
            "eip155:1313161554".into(),
            (
                "https://mainnet.aurora.dev".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Aurora Testnet
        (
            "eip155:1313161555".into(),
            (
                "https://testnet.aurora.dev".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
    ])
}
