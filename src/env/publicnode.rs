use {crate::providers::Weight, serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PublicnodeConfig {
    #[serde(default = "default_supported_chains")]
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for PublicnodeConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Ethereum mainnet
        ("eip155:1".into(), ("ethereum".into(), Weight(1.0))),
        // Ethereum goerli
        ("eip155:5".into(), ("ethereum-goerli".into(), Weight(10.0))),
        // Binance Smart Chain mainnet
        ("eip155:56".into(), ("bsc".into(), Weight(3.0))),
        // Binance Smart Chain testnet
        ("eip155:97".into(), ("bsc-testnet".into(), Weight(10.0))),
        // Avalanche c chain
        (
            "eip155:43114".into(),
            ("avalanche-c-chain".into(), Weight(1.0)),
        ),
        // Avalanche fuji testnet
        (
            "eip155:43113".into(),
            ("avalanche-fuji-c-chain".into(), Weight(10.0)),
        ),
        // Polygon bor mainnet
        ("eip155:137".into(), ("polygon-bor".into(), Weight(1.0))),
        // Polygon bor testnet
        (
            "eip155:80001".into(),
            ("polygon-mumbai-bor".into(), Weight(10.0)),
        ),
    ])
}
