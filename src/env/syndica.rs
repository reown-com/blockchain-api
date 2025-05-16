use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct SyndicaConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
    pub supported_ws_chains: HashMap<String, (String, Weight)>,
    pub api_key: String,
}

impl SyndicaConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            supported_chains: default_supported_chains(),
            supported_ws_chains: default_ws_supported_chains(),
            api_key,
        }
    }
}

impl ProviderConfig for SyndicaConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_ws_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Syndica
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Solana Mainnet
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".into(),
            (
                "https://solana-mainnet.api.syndica.io".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Solana Devnet
        (
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1".into(),
            (
                "https://solana-devnet.api.syndica.io".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}

fn default_ws_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Solana Mainnet
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".into(),
            (
                "wss://solana-mainnet.api.syndica.io".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Solana Devnet
        (
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1".into(),
            (
                "wss://solana-devnet.api.syndica.io".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
