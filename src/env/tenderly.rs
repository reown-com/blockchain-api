use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct TenderlyConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
    pub supported_ws_chains: HashMap<String, (String, Weight)>,
}

impl Default for TenderlyConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
            supported_ws_chains: default_ws_supported_chains(),
        }
    }
}

impl ProviderConfig for TenderlyConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_ws_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Tenderly
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum Mainnet
        (
            "eip155:1".into(),
            ("mainnet".into(), Weight::new(Priority::Max).unwrap()),
        ),
        // Ethereum Görli
        (
            "eip155:5".into(),
            ("goerli".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Ethereum Sepolia
        (
            "eip155:11155111".into(),
            ("sepolia".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism Mainnet
        (
            "eip155:10".into(),
            ("optimism".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism Görli
        (
            "eip155:420".into(),
            (
                "optimism-goerli".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Polygon Mainnet
        (
            "eip155:137".into(),
            ("polygon".into(), Weight::new(Priority::High).unwrap()),
        ),
        // Polygon Mumbai
        (
            "eip155:80001".into(),
            (
                "polygon-mumbai".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Base Mainnet
        (
            "eip155:8453".into(),
            ("base".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Base Görli
        (
            "eip155:84531".into(),
            ("base-goerli".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Boba Ethereum Mainnet
        (
            "eip155:288".into(),
            (
                "boba-ethereum".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Boba BNB Mainnet
        (
            "eip155:56288".into(),
            ("boba-bnb".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Boba BNB Testnet
        (
            "eip155:9728".into(),
            (
                "boba-bnb-testnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}

fn default_ws_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum Mainnet
        (
            "eip155:1".into(),
            ("mainnet".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Ethereum Görli
        (
            "eip155:5".into(),
            ("goerli".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Ethereum Sepolia
        (
            "eip155:11155111".into(),
            ("sepolia".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism Mainnet
        (
            "eip155:10".into(),
            ("optimism".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Optimism Görli
        (
            "eip155:420".into(),
            (
                "optimism-goerli".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Polygon Mainnet
        (
            "eip155:137".into(),
            ("polygon".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Polygon Mumbai
        (
            "eip155:80001".into(),
            (
                "polygon-mumbai".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Base Mainnet
        (
            "eip155:8453".into(),
            ("base".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Base Görli
        (
            "eip155:84531".into(),
            ("base-goerli".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Boba Ethereum Mainnet
        (
            "eip155:288".into(),
            (
                "boba-ethereum".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Boba BNB Mainnet
        (
            "eip155:56288".into(),
            ("boba-bnb".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Boba BNB Testnet
        (
            "eip155:9728".into(),
            (
                "boba-bnb-testnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
