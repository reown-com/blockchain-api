use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct ZanConfig {
    pub api_key: String,
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl ZanConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for ZanConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Zan
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md

    HashMap::from([
        // Ethereum mainnet
        (
            "eip155:1".into(),
            (
                "eth/mainnet".into(),
                Weight::new(Priority::Minimal).unwrap(),
            ),
        ),
        // BSC mainnet
        (
            "eip155:56".into(),
            ("bsc/mainnet".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Polygon mainnet
        (
            "eip155:137".into(),
            (
                "polygon/mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Optimism mainnet
        (
            "eip155:10".into(),
            ("opt/mainnet".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Arbitrum mainnet
        (
            "eip155:42161".into(),
            ("arb/one".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Base mainnet
        (
            "eip155:8453".into(),
            (
                "base/mainnet".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // Sui
        (
            "sui:mainnet".into(),
            ("sui/mainnet".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Bitcoin mainnet
        (
            "bip122:000000000019d6689c085ae165831e93".into(),
            ("btc/mainnet".into(), Weight::new(Priority::Normal).unwrap()),
        ),
        // Bitcoin testnet
        (
            "bip122:000000000933ea01ad0ee984209779ba".into(),
            ("btc/testnet".into(), Weight::new(Priority::Normal).unwrap()),
        ),
    ])
}
