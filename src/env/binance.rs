use {crate::providers::Weight, serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct BinanceConfig {
    pub project_id: String,

    #[serde(default = "default_binance_supported_chains")]
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self {
            project_id: Default::default(),
            supported_chains: default_binance_supported_chains(),
        }
    }
}

fn default_binance_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Binance Smart Chain Mainnet
        (
            "eip155:56".into(),
            ("https://bsc-dataseed.binance.org/".into(), Weight(1.0)),
        ),
        // Binance Smart Chain Testnet
        (
            "eip155:97".into(),
            (
                "https://data-seed-prebsc-1-s1.binance.org:8545".into(),
                Weight(1.0),
            ),
        ),
    ])
}
