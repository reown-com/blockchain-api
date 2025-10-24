use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct HiroConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for HiroConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for HiroConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Hiro
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Stacks Mainnet
        (
            "stacks:1".into(),
            (
                "https://api.mainnet.hiro.so/".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Stacks Testnet
        (
            "stacks:2147483648".into(),
            (
                "https://api.testnet.hiro.so/".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
