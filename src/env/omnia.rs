use {crate::providers::Weight, serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct OmniatechConfig {
    #[serde(default = "default_supported_chains")]
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for OmniatechConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Ethereum mainnet
        ("eip155:1".into(), ("eth".into(), Weight(1.0))),
        // Binance Smart Chain mainnet
        ("eip155:56".into(), ("bsc".into(), Weight(1.0))),
        // Polygon
        ("eip155:137".into(), ("matic".into(), Weight(1.0))),
        // Near
        ("near".into(), ("near".into(), Weight(1.0))),
        // Aurora
        ("eip155:1313161554".into(), ("aurora".into(), Weight(1.0))),
        // Optimism
        ("eip155:10".into(), ("op".into(), Weight(1.0))),
        // Solana
        ("solana-mainnet".into(), ("sol".into(), Weight(1.0))),
        // Avalanche C chain
        ("eip155:43114".into(), ("avax".into(), Weight(1.0))),
    ])
}
