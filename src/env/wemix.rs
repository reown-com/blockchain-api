use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct WemixConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for WemixConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for WemixConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Wemix
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Wemix Mainnet
        (
            "eip155:1111".into(),
            (
                "https://api.wemix.com/".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Wemix Testnet
        (
            "eip155:1112".into(),
            (
                "https://api.test.wemix.com".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
