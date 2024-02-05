use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct QuicknodeConfig {
    pub api_token: String,
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl QuicknodeConfig {
    pub fn new(api_token: String) -> Self {
        Self {
            api_token,
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for QuicknodeConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Quicknode
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // zkSync
        (
            "eip155:324".into(),
            (
                "snowy-chaotic-hill.zksync-mainnet".into(),
                Weight::new(Priority::High).unwrap(),
            ),
        ),
    ])
}
