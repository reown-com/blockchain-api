use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct CallStaticConfig {
    pub api_key: String,
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl CallStaticConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for CallStaticConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::CallStatic
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // BSC mainnet
        (
            "eip155:56".into(),
            ("bsc".into(), Weight::new(Priority::Normal).unwrap()),
        ),
    ])
}
