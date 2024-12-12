use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct MorphConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for MorphConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for MorphConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Morph
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Morph Mainnet
        (
            "eip155:2818".into(),
            (
                "rpc-quicknode".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Morph Holesky
        (
            "eip155:2810".into(),
            (
                "rpc-quicknode-holesky".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
