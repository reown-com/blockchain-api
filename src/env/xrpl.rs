use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct XrplConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for XrplConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for XrplConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Xrpl
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // XRPL EVM Mainnet
        (
            "eip155:1440000".into(),
            (
                "https://rpc.xrplevm.org".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // XRPL EVM Testnet
        (
            "eip155:1449000".into(),
            (
                "https://rpc.testnet.xrplevm.org".into(),
                Weight::new(Priority::Low).unwrap(),
            ),
        ),
    ])
}
