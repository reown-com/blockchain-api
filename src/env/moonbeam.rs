use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct MoonbeamConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for MoonbeamConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for MoonbeamConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Moonbeam
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Moonbeam Mainnet
        (
            "eip155:1284".into(),
            (
                "https://rpc.api.moonbeam.network".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
