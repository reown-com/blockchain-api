use {super::ProviderConfig, crate::providers::Weight, std::collections::HashMap};

#[derive(Debug)]
pub struct OmniatechConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for OmniatechConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for OmniatechConfig {
    fn supported_chains(&self) -> &HashMap<String, (String, Weight)> {
        &self.supported_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Omniatech
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Ethereum mainnet
        ("eip155:1".into(), ("eth".into(), Weight(1.into()))),
        // Binance Smart Chain mainnet
        ("eip155:56".into(), ("bsc".into(), Weight(1.into()))),
        // Polygon
        ("eip155:137".into(), ("matic".into(), Weight(1.into()))),
        // Near
        ("near".into(), ("near".into(), Weight(1.into()))),
        // Aurora
        (
            "eip155:1313161554".into(),
            ("aurora".into(), Weight(1.into())),
        ),
        // Optimism
        ("eip155:10".into(), ("op".into(), Weight(1.into()))),
        // Solana
        ("solana-mainnet".into(), ("sol".into(), Weight(1.into()))),
        // Avalanche C chain
        ("eip155:43114".into(), ("avax".into(), Weight(1.into()))),
    ])
}
