use {super::ProviderConfig, crate::providers::Weight, std::collections::HashMap};

#[derive(Debug)]
pub struct PoktConfig {
    pub project_id: String,

    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl PoktConfig {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for PoktConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Pokt
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Solana Mainnet
        (
            "solana:4sgjmw1sunhzsxgspuhpqldx6wiyjntz".into(),
            ("solana-mainnet".into(), Weight(1.into())),
        ),
        // Avax C-Chain
        (
            "eip155:43114".into(),
            ("avax-mainnet".into(), Weight(1.into())),
        ),
        // Gnosis
        ("eip155:100".into(), ("poa-xdai".into(), Weight(1.into()))),
        // Binance Smart Chain
        ("eip155:56".into(), ("bsc-mainnet".into(), Weight(5.into()))),
        // Ethereum
        ("eip155:1".into(), ("mainnet".into(), Weight(40.into()))),
        ("eip155:5".into(), ("goerli".into(), Weight(1.into()))),
        // Optimism
        (
            "eip155:10".into(),
            ("optimism-mainnet".into(), Weight(1.into())),
        ),
        // Arbitrum
        (
            "eip155:42161".into(),
            ("arbitrum-mainnet".into(), Weight(1.into())),
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
    ])
}
