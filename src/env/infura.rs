use {serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InfuraConfig {
    pub project_id: String,

    #[serde(default = "default_infura_supported_chains")]
    pub supported_chains: HashMap<String, String>,

    #[serde(default = "default_infura_ws_supported_chains")]
    pub supported_ws_chains: HashMap<String, String>,
}

fn default_infura_supported_chains() -> HashMap<String, String> {
    HashMap::from([
        // Ethereum
        ("eip155:1".into(), "mainnet".into()),
        ("eip155:3".into(), "ropsten".into()),
        ("eip155:42".into(), "kovan".into()),
        ("eip155:4".into(), "rinkeby".into()),
        ("eip155:5".into(), "goerli".into()),
        // Polygon
        ("eip155:137".into(), "polygon-mainnet".into()),
        ("eip155:80001".into(), "polygon-mumbai".into()),
        // Optimism
        ("eip155:10".into(), "optimism-mainnet".into()),
        ("eip155:69".into(), "optimism-kovan".into()),
        ("eip155:420".into(), "optimism-goerli".into()),
        // Arbitrum
        ("eip155:42161".into(), "arbitrum-mainnet".into()),
        ("eip155:421611".into(), "arbitrum-rinkeby".into()),
        ("eip155:421613".into(), "arbitrum-goerli".into()),
        // Celo
        ("eip155:42220".into(), "celo-mainnet".into()),
        // Aurora
        ("eip155:1313161554".into(), "aurora-mainnet".into()),
        ("eip155:1313161555".into(), "aurora-testnet".into()),
    ])
}

fn default_infura_ws_supported_chains() -> HashMap<String, String> {
    HashMap::from([
        // Ethereum
        ("eip155:1".into(), "mainnet".into()),
        ("eip155:3".into(), "ropsten".into()),
        ("eip155:42".into(), "kovan".into()),
        ("eip155:4".into(), "rinkeby".into()),
        ("eip155:5".into(), "goerli".into()),
        // Polygon
        ("eip155:137".into(), "polygon-mainnet".into()),
        ("eip155:80001".into(), "polygon-mumbai".into()),
        // Optimism
        ("eip155:10".into(), "optimism-mainnet".into()),
        ("eip155:69".into(), "optimism-kovan".into()),
        ("eip155:420".into(), "optimism-goerli".into()),
        // Arbitrum
        ("eip155:42161".into(), "arbitrum-mainnet".into()),
        ("eip155:421611".into(), "arbitrum-rinkeby".into()),
        ("eip155:421613".into(), "arbitrum-goerli".into()),
        // Celo
        ("eip155:42220".into(), "celo-mainnet".into()),
        // Aurora
        ("eip155:1313161554".into(), "aurora-mainnet".into()),
        ("eip155:1313161555".into(), "aurora-testnet".into()),
    ])
}
