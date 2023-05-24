use {crate::providers::Weight, std::collections::HashMap};

#[derive(Debug)]
pub struct BinanceConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Binance Smart Chain Mainnet
        (
            "eip155:56".into(),
            ("https://bsc-dataseed.binance.org/".into(), Weight(5.into())),
        ),
        // Binance Smart Chain Testnet
        (
            "eip155:97".into(),
            (
                "https://data-seed-prebsc-1-s1.binance.org:8545".into(),
                Weight(3.into()),
            ),
        ),
    ])
}
