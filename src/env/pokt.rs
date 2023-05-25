use {super::ProviderConfig, crate::providers::Weight, std::collections::HashMap};

#[derive(Debug, Clone)]
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
    fn supported_chains(&self) -> &HashMap<String, (String, Weight)> {
        &self.supported_chains
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
        ("eip155:56".into(), ("bsc-mainnet".into(), Weight(3.into()))),
        // ETH Mainnet
        ("eip155:56".into(), ("bsc-mainnet".into(), Weight(1.into()))),
    ])
}
