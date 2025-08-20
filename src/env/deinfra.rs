use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct DeInfraConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for DeInfraConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for DeInfraConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::DeInfra
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // DeInfra Mainnet
        (
            "eip155:100501".into(),
            (
                "https://c100501n3.deinfra.net/jsonrpc".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // DeInfra Devnet3
        (
            "eip155:1000000003".into(),
            (
                "https://c3n1.thepower.io/jsonrpc".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
