use {crate::providers::Weight, std::collections::HashMap};

#[derive(Debug)]
pub struct PublicnodeConfig {
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
        ("eip155:1".into(), ("ethereum".into(), Weight(1.into()))),
        // Ethereum goerli
        (
            "eip155:5".into(),
            ("ethereum-goerli".into(), Weight(10.into())),
        ),
        // Binance Smart Chain mainnet
        ("eip155:56".into(), ("bsc".into(), Weight(3.into()))),
        // Binance Smart Chain testnet
        (
            "eip155:97".into(),
            ("bsc-testnet".into(), Weight(10.into())),
        ),
        // Avalanche c chain
        (
            "eip155:43114".into(),
            ("avalanche-c-chain".into(), Weight(1.into())),
        ),
        // Avalanche fuji testnet
        (
            "eip155:43113".into(),
            ("avalanche-fuji-c-chain".into(), Weight(10.into())),
        ),
        // Polygon bor mainnet
        (
            "eip155:137".into(),
            ("polygon-bor".into(), Weight(1.into())),
        ),
        // Polygon bor testnet
        (
            "eip155:80001".into(),
            ("polygon-mumbai-bor".into(), Weight(10.into())),
        ),
    ])
}
