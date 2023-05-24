use {crate::providers::Weight, std::collections::HashMap};

#[derive(Debug)]
pub struct ZKSyncConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for ZKSyncConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // zkSync Testnet
        (
            "eip155:280".into(),
            (
                "https://zksync2-testnet.zksync.dev".into(),
                Weight(1.into()),
            ),
        ),
        // zkSync Mainnet
        (
            "eip155:324".into(),
            ("https://zksync2-mainnet.zksync.io".into(), Weight(1.into())),
        ),
    ])
}
