use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct PublicnodeConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for PublicnodeConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for PublicnodeConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Publicnode
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum mainnet
        (
            "eip155:1".into(),
            ("ethereum".into(), Weight::new(Priority::High).unwrap()),
        ),
        // Ethereum Holesky
        (
            "eip155:17000".into(),
            (
                "ethereum-holesky-rpc".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Base mainnet
        (
            "eip155:8453".into(),
            ("base".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Binance Smart Chain mainnet
        (
            "eip155:56".into(),
            ("bsc".into(), Weight::new(Priority::High).unwrap()),
        ),
        // Binance Smart Chain testnet
        (
            "eip155:97".into(),
            ("bsc-testnet".into(), Weight::new(Priority::High).unwrap()),
        ),
        // Avalanche c chain
        (
            "eip155:43114".into(),
            (
                "avalanche-c-chain".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Avalanche fuji testnet
        (
            "eip155:43113".into(),
            (
                "avalanche-fuji-c-chain".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Polygon bor mainnet
        (
            "eip155:137".into(),
            ("polygon-bor".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Mantle mainnet
        (
            "eip155:5000".into(),
            ("mantle-rpc".into(), Weight::new(Priority::High).unwrap()),
        ),
    ])
}
