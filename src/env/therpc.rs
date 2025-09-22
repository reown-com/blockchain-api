use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct TheRpcConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for TheRpcConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for TheRpcConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::TheRpc
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Arbitrum One
        (
            "eip155:42161".into(),
            ("arbitrum".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // BSC
        (
            "eip155:56".into(),
            ("bsc".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Polygon
        (
            "eip155:137".into(),
            ("polygon".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism
        (
            "eip155:10".into(),
            ("optimism".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Base
        (
            "eip155:8453".into(),
            ("base".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Sonic
        (
            "eip155:146".into(),
            ("sonic".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Unichain Sepolia
        (
            "eip155:1301".into(),
            (
                "unichain-sepolia".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Unichain Mainnet
        (
            "eip155:130".into(),
            ("unichain".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Bitcoin mainnet
        (
            "bip122:000000000019d6689c085ae165831e93".into(),
            ("bitcoin".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Bitcoin testnet
        (
            "bip122:000000000933ea01ad0ee984209779ba".into(),
            (
                "bitcoin-testnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Solana mainnet
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".into(),
            ("solana".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Solana devnet
        (
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1".into(),
            (
                "solana-devnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
