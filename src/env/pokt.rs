use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct PoktConfig {
    pub project_id: String,

    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl PoktConfig {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for PoktConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Pokt
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Solana Mainnet
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".into(),
            (
                "solana-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        (
            // Incorrect (not CAIP-2), uses block explorer blockhash instead of getGenesisHash RPC
            "solana:4sgjmw1sunhzsxgspuhpqldx6wiyjntz".into(),
            (
                "solana-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // AVAX mainnet
        (
            "eip155:43114".into(),
            (
                "avax-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Gnosis
        (
            "eip155:100".into(),
            (
                "gnosischain-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Base mainnet
        (
            "eip155:8453".into(),
            (
                "base-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Base Sepolia
        (
            "eip155:84532".into(),
            (
                "base-testnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Binance Smart Chain
        (
            "eip155:56".into(),
            ("bsc-mainnet".into(), Weight::new(Priority::Max).unwrap()),
        ),
        // Ethereum mainnet
        (
            "eip155:1".into(),
            ("eth-mainnet".into(), Weight::new(Priority::Max).unwrap()),
        ),
        // Ethereum holesky
        (
            "eip155:17000".into(),
            (
                "holesky-fullnode-testnet".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Ethereum sepolia
        (
            "eip155:11155111".into(),
            ("sepolia".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism
        (
            "eip155:10".into(),
            (
                "optimism-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Arbitrum
        (
            "eip155:42161".into(),
            (
                "arbitrum-one".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Polygon
        (
            "eip155:137".into(),
            ("poly-mainnet".into(), Weight::new(Priority::High).unwrap()),
        ),
        (
            "eip155:1101".into(),
            (
                "polygon-zkevm-mainnet".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        (
            "eip155:80002".into(),
            (
                "amoy-testnet-archival".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Celo
        (
            "eip155:42220".into(),
            (
                "celo-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Klaytn
        (
            "eip155:8217".into(),
            (
                "klaytn-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Near protocol
        (
            "near:mainnet".into(),
            (
                "near-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
