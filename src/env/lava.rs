use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct LavaConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
    pub api_key: String,
}

impl LavaConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            supported_chains: default_supported_chains(),
            api_key,
        }
    }
}

impl ProviderConfig for LavaConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Lava
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum Mainnet
        (
            "eip155:1".into(),
            ("eth".into(), Weight::new(Priority::Low).unwrap()),
        ),
        // Ethereum Sepolia
        (
            "eip155:11155111".into(),
            ("sep1".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Ethereum Holesky
        (
            "eip155:17000".into(),
            ("hol1".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Base Mainnet
        (
            "eip155:8453".into(),
            ("base".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Arbitrum One
        (
            "eip155:42161".into(),
            ("arb1".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Arbitrum Sepolia
        (
            "eip155:421614".into(),
            ("arbs".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Solana Mainnet
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".into(),
            ("solana".into(), Weight::new(Priority::Normal).unwrap()),
        ),
    ])
}
