use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct BinanceConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl ProviderConfig for BinanceConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Binance
    }
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Binance Smart Chain Mainnet
        (
            "eip155:56".into(),
            (
                "https://bsc-dataseed.binance.org/".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Binance Smart Chain Testnet
        (
            "eip155:97".into(),
            (
                "https://data-seed-prebsc-1-s1.binance.org:8545".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
    ])
}
