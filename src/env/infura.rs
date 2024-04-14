use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct InfuraConfig {
    pub project_id: String,

    pub supported_chains: HashMap<String, (String, Weight)>,

    pub supported_ws_chains: HashMap<String, (String, Weight)>,
}

impl InfuraConfig {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            supported_chains: default_supported_chains(),
            supported_ws_chains: default_ws_supported_chains(),
        }
    }
}

impl ProviderConfig for InfuraConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_ws_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Infura
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum
        (
            "eip155:1".into(),
            ("mainnet".into(), Weight::new(Priority::Max).unwrap()),
        ),
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
        (
            "eip155:11155420".into(),
            (
                "optimism-sepolia".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Arbitrum
        (
            "eip155:42161".into(),
            (
                "arbitrum-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        (
            "eip155:421614".into(),
            (
                "arbitrum-sepolia".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Polygon
        (
            "eip155:137".into(),
            (
                "polygon-mainnet".into(),
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
        // Linea Mainnet
        (
            "eip155:59144".into(),
            (
                "linea-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}

fn default_ws_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum
        (
            "eip155:1".into(),
            ("mainnet".into(), Weight::new(Priority::Normal).unwrap()),
        ),
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
        (
            "eip155:11155420".into(),
            (
                "optimism-sepolia".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Arbitrum
        (
            "eip155:42161".into(),
            (
                "arbitrum-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        (
            "eip155:421614".into(),
            (
                "arbitrum-sepolia".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
