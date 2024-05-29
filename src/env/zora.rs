use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct ZoraConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
    pub supported_ws_chains: HashMap<String, (String, Weight)>,
}

impl Default for ZoraConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
            supported_ws_chains: default_ws_supported_chains(),
        }
    }
}

impl ProviderConfig for ZoraConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_ws_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Zora
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Zora Mainnet
        (
            "eip155:7777777".into(),
            (
                "https://rpc.zora.energy".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Zora Sepolia
        (
            "eip155:999999999".into(),
            (
                "https://sepolia.rpc.zora.energy".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}

fn default_ws_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Zora Mainnet
        (
            "eip155:7777777".into(),
            (
                "wss://rpc.zora.energy".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
