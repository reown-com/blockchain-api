use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct SuiConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for SuiConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for SuiConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Sui
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Sui mainnet
        (
            "sui:mainnet".into(),
            (
                "https://fullnode.mainnet.sui.io".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Sui testnet
        (
            "sui:testnet".into(),
            (
                "https://fullnode.testnet.sui.io".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Sui devnet
        (
            "sui:devnet".into(),
            (
                "https://fullnode.devnet.sui.io".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
