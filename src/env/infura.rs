use {super::ProviderConfig, crate::providers::Weight, std::collections::HashMap};

#[derive(Debug, Clone)]
pub struct InfuraConfig {
    pub project_id: String,

    pub supported_chains: HashMap<String, (String, Weight)>,

    pub supported_ws_chains: HashMap<String, (String, Weight)>,
}

impl InfuraConfig {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            supported_chains: default_supported_chains(),
            supported_ws_chains: default_ws_supported_chains(),
        }
    }
}

impl ProviderConfig for InfuraConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Infura
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Ethereum
        ("eip155:1".into(), ("mainnet".into(), Weight(10.into()))),
        ("eip155:3".into(), ("ropsten".into(), Weight(1.into()))),
        ("eip155:42".into(), ("kovan".into(), Weight(1.into()))),
        ("eip155:4".into(), ("rinkeby".into(), Weight(1.into()))),
        ("eip155:5".into(), ("goerli".into(), Weight(1.into()))),
        // Optimism
        (
            "eip155:10".into(),
            ("optimism-mainnet".into(), Weight(1.into())),
        ),
        (
            "eip155:69".into(),
            ("optimism-kovan".into(), Weight(1.into())),
        ),
        (
            "eip155:420".into(),
            ("optimism-goerli".into(), Weight(1.into())),
        ),
        // Arbitrum
        (
            "eip155:42161".into(),
            ("arbitrum-mainnet".into(), Weight(1.into())),
        ),
        (
            "eip155:421611".into(),
            ("arbitrum-rinkeby".into(), Weight(1.into())),
        ),
        (
            "eip155:421613".into(),
            ("arbitrum-goerli".into(), Weight(1.into())),
        ),
        // Polygon
        (
            "eip155:137".into(),
            ("polygon-mainnet".into(), Weight(5.into())),
        ),
        (
            "eip155:80001".into(),
            ("polygon-mumbai".into(), Weight(1.into())),
        ),
        // Celo
        (
            "eip155:42220".into(),
            ("celo-mainnet".into(), Weight(1.into())),
        ),
        // Aurora
        (
            "eip155:1313161554".into(),
            ("aurora-mainnet".into(), Weight(1.into())),
        ),
        (
            "eip155:1313161555".into(),
            ("aurora-testnet".into(), Weight(1.into())),
        ),
    ])
}

fn default_ws_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Ethereum
        ("eip155:1".into(), ("mainnet".into(), Weight(1.into()))),
        ("eip155:3".into(), ("ropsten".into(), Weight(1.into()))),
        ("eip155:42".into(), ("kovan".into(), Weight(1.into()))),
        ("eip155:4".into(), ("rinkeby".into(), Weight(1.into()))),
        ("eip155:5".into(), ("goerli".into(), Weight(1.into()))),
        // Optimism
        (
            "eip155:10".into(),
            ("optimism-mainnet".into(), Weight(1.into())),
        ),
        (
            "eip155:69".into(),
            ("optimism-kovan".into(), Weight(1.into())),
        ),
        (
            "eip155:420".into(),
            ("optimism-goerli".into(), Weight(1.into())),
        ),
        // Arbitrum
        (
            "eip155:42161".into(),
            ("arbitrum-mainnet".into(), Weight(1.into())),
        ),
        (
            "eip155:421611".into(),
            ("arbitrum-rinkeby".into(), Weight(1.into())),
        ),
        (
            "eip155:421613".into(),
            ("arbitrum-goerli".into(), Weight(1.into())),
        ),
        // Celo
        (
            "eip155:42220".into(),
            ("celo-mainnet".into(), Weight(1.into())),
        ),
        // Aurora
        (
            "eip155:1313161554".into(),
            ("aurora-mainnet".into(), Weight(1.into())),
        ),
        (
            "eip155:1313161555".into(),
            ("aurora-testnet".into(), Weight(1.into())),
        ),
    ])
}
