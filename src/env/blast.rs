use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct BlastConfig {
    pub api_key: String,
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl BlastConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for BlastConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Blast
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Rootstock mainnet
        (
            "eip155:30".into(),
            (
                "rootstock-mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Rootstock testnet
        (
            "eip155:31".into(),
            (
                "rootstock-testnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Polygon Mainnet
        (
            "eip155:137".into(),
            (
                "polygon-mainnet".into(),
                Weight::new(Priority::Low).unwrap(),
            ),
        ),
        // Polygon Amoy
        (
            "eip155:80002".into(),
            ("polygon-amoy".into(), Weight::new(Priority::Low).unwrap()),
        ),
    ])
}
