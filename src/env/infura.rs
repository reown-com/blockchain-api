use {crate::providers::Weight, serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct InfuraConfig {
    pub project_id: String,

    #[serde(default = "default_supported_chains")]
    pub supported_chains: HashMap<String, (String, Weight)>,

    #[serde(default = "default_ws_supported_chains")]
    pub supported_ws_chains: HashMap<String, (String, Weight)>,
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Ethereum
        ("eip155:1".into(), ("mainnet".into(), Weight(10.0))),
        ("eip155:3".into(), ("ropsten".into(), Weight(1.0))),
        ("eip155:42".into(), ("kovan".into(), Weight(1.0))),
        ("eip155:4".into(), ("rinkeby".into(), Weight(1.0))),
        ("eip155:5".into(), ("goerli".into(), Weight(1.0))),
        // Optimism
        ("eip155:10".into(), ("optimism-mainnet".into(), Weight(1.0))),
        ("eip155:69".into(), ("optimism-kovan".into(), Weight(1.0))),
        ("eip155:420".into(), ("optimism-goerli".into(), Weight(1.0))),
        // Arbitrum
        (
            "eip155:42161".into(),
            ("arbitrum-mainnet".into(), Weight(1.0)),
        ),
        (
            "eip155:421611".into(),
            ("arbitrum-rinkeby".into(), Weight(1.0)),
        ),
        (
            "eip155:421613".into(),
            ("arbitrum-goerli".into(), Weight(1.0)),
        ),
        // Polygon
        ("eip155:137".into(), ("polygon-mainnet".into(), Weight(5.0))),
        (
            "eip155:80001".into(),
            ("polygon-mumbai".into(), Weight(1.0)),
        ),
        // Celo
        ("eip155:42220".into(), ("celo-mainnet".into(), Weight(1.0))),
        // Aurora
        (
            "eip155:1313161554".into(),
            ("aurora-mainnet".into(), Weight(1.0)),
        ),
        (
            "eip155:1313161555".into(),
            ("aurora-testnet".into(), Weight(1.0)),
        ),
    ])
}

fn default_ws_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Ethereum
        ("eip155:1".into(), ("mainnet".into(), Weight(1.0))),
        ("eip155:3".into(), ("ropsten".into(), Weight(1.0))),
        ("eip155:42".into(), ("kovan".into(), Weight(1.0))),
        ("eip155:4".into(), ("rinkeby".into(), Weight(1.0))),
        ("eip155:5".into(), ("goerli".into(), Weight(1.0))),
        // Optimism
        ("eip155:10".into(), ("optimism-mainnet".into(), Weight(1.0))),
        ("eip155:69".into(), ("optimism-kovan".into(), Weight(1.0))),
        ("eip155:420".into(), ("optimism-goerli".into(), Weight(1.0))),
        // Arbitrum
        (
            "eip155:42161".into(),
            ("arbitrum-mainnet".into(), Weight(1.0)),
        ),
        (
            "eip155:421611".into(),
            ("arbitrum-rinkeby".into(), Weight(1.0)),
        ),
        (
            "eip155:421613".into(),
            ("arbitrum-goerli".into(), Weight(1.0)),
        ),
        // Celo
        ("eip155:42220".into(), ("celo-mainnet".into(), Weight(1.0))),
        // Aurora
        (
            "eip155:1313161554".into(),
            ("aurora-mainnet".into(), Weight(1.0)),
        ),
        (
            "eip155:1313161555".into(),
            ("aurora-testnet".into(), Weight(1.0)),
        ),
    ])
}
