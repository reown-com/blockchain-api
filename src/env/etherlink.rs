use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct EtherlinkConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for EtherlinkConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for EtherlinkConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Etherlink
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Etherlink Mainnet
        (
            "eip155:42793".into(),
            (
                "https://node.mainnet.etherlink.com".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Etherlink Testnet
        (
            "eip155:128123".into(),
            (
                "https://node.ghostnet.etherlink.com".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}