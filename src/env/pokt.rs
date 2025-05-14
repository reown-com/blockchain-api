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
            ("solana".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        (
            // Incorrect (not CAIP-2), uses block explorer blockhash instead of getGenesisHash RPC
            "solana:4sgjmw1sunhzsxgspuhpqldx6wiyjntz".into(),
            ("solana".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // AVAX mainnet
        (
            "eip155:43114".into(),
            ("avax".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Gnosis
        (
            "eip155:100".into(),
            ("gnosis".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Base mainnet
        (
            "eip155:8453".into(),
            ("base".into(), Weight::new(Priority::Normal).unwrap()),
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
            ("bsc".into(), Weight::new(Priority::Max).unwrap()),
        ),
        // Ethereum mainnet
        (
            "eip155:1".into(),
            ("eth".into(), Weight::new(Priority::Minimal).unwrap()),
        ),
        // Ethereum holesky
        (
            "eip155:17000".into(),
            (
                "eth-holesky-testnet".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Ethereum sepolia
        (
            "eip155:11155111".into(),
            (
                "eth-sepolia-testnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Optimism
        (
            "eip155:10".into(),
            ("optimism".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism Sepolia
        // TODO: Temporary disabled due to issues with the provider
        (
            "eip155:11155420".into(),
            (
                "optimism-sepolia-testnet".into(),
                Weight::new(Priority::Disabled).unwrap(),
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
        // Arbitrum Sepolia
        (
            "eip155:421614".into(),
            (
                "arbitrum-sepolia-testnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Polygon
        (
            "eip155:137".into(),
            ("polygon".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        (
            "eip155:1101".into(),
            ("polygon-zkevm".into(), Weight::new(Priority::High).unwrap()),
        ),
        (
            "eip155:80002".into(),
            (
                "polygon-amoy-testnet".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
        // Celo
        (
            "eip155:42220".into(),
            ("celo".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Linea
        (
            "eip155:59144".into(),
            ("linea".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Kaia Mainnet
        (
            "eip155:8217".into(),
            ("kaia".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // zkSync
        (
            "eip155:324".into(),
            ("zksync-era".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Scroll
        (
            "eip155:534352".into(),
            ("scroll".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Berachain Mainnet
        (
            "eip155:80094".into(),
            ("berachain".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Sonic Mainnet
        (
            "eip155:146".into(),
            ("sonic".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Near protocol
        (
            "near:mainnet".into(),
            ("near".into(), Weight::new(Priority::Normal).unwrap()),
        ),
    ])
}
