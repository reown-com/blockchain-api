use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct EdexaConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for EdexaConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for EdexaConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Edexa
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // edeXa Mainnet
        (
            "eip155:5424".into(),
            (
                "https://rpc.edexa.network".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // edeXa Testnet
        (
            "eip155:1995".into(),
            (
                "https://rpc.testnet.edexa.network".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
