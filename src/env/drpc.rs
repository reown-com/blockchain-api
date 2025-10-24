use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct DrpcConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for DrpcConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for DrpcConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Drpc
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum Mainnet
        (
            "eip155:1".into(),
            (
                "https://eth.drpc.org/".into(),
                Weight::new(Priority::Minimal).unwrap(),
            ),
        ),
        // Ethereum Sepolia
        (
            "eip155:11155111".into(),
            (
                "https://sepolia.drpc.org".into(),
                Weight::new(Priority::Minimal).unwrap(),
            ),
        ),
        // Ethereum Holesky
        (
            "eip155:17000".into(),
            (
                "https://holesky.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Ethereum Hoodi
        (
            "eip155:560048".into(),
            (
                "https://hoodi.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Arbitrum One
        (
            "eip155:42161".into(),
            (
                "https://arbitrum.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Base
        (
            "eip155:8453".into(),
            (
                "https://base.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // BSC
        (
            "eip155:56".into(),
            (
                "https://bsc.drpc.org".into(),
                Weight::new(Priority::Minimal).unwrap(),
            ),
        ),
        // Polygon
        (
            "eip155:137".into(),
            (
                "https://polygon.drpc.org".into(),
                Weight::new(Priority::Minimal).unwrap(),
            ),
        ),
        // Optimism
        (
            "eip155:10".into(),
            (
                "https://optimism.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Unichain
        (
            "eip155:1301".into(),
            (
                "https://unichain-sepolia.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Kaia / Klaytn
        (
            "eip155:8217".into(),
            (
                "https://klaytn.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Berachain Mainnet
        (
            "eip155:80094".into(),
            (
                "https://berachain.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Monad Testnet
        (
            "eip155:10143".into(),
            (
                "https://monad-testnet.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Sonic Mainnet
        (
            "eip155:146".into(),
            (
                "https://sonic.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Sonic Testnet
        (
            "eip155:57054".into(),
            (
                "https://sonic-testnet.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Linea Mainnet
        (
            "eip155:59144".into(),
            (
                "https://linea.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Celo Mainnet
        (
            "eip155:42220".into(),
            (
                "https://celo.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Sei Mainnet
        (
            "eip155:1329".into(),
            (
                "https://sei.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Polygon Amoy
        (
            "eip155:80002".into(),
            (
                "https://polygon-amoy.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Avalanche Fuji
        (
            "eip155:43113".into(),
            (
                "https://avalanche-fuji.drpc.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
