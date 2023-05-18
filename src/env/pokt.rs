use {crate::providers::Weight, serde::Deserialize, std::collections::HashMap};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PoktConfig {
    pub project_id: String,

    #[serde(default = "default_pokt_supported_chains")]
    pub supported_chains: HashMap<String, (String, Weight)>,
}

fn default_pokt_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([
        // Solana Mainnet
        (
            "solana:4sgjmw1sunhzsxgspuhpqldx6wiyjntz".into(),
            ("solana-mainnet".into(), Weight(1.0)),
        ),
        // Avax C-Chain
        ("eip155:43114".into(), ("avax-mainnet".into(), Weight(1.0))),
        // Gnosis
        ("eip155:100".into(), ("poa-xdai".into(), Weight(1.0))),
        // Binance Smart Chain
        ("eip155:56".into(), ("bsc-mainnet".into(), Weight(1.0))),
    ])
}
