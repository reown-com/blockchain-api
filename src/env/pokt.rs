use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PoktConfig {
    pub project_id: String,

    #[serde(default = "default_pokt_supported_chains")]
    pub supported_chains: HashMap<String, String>,
}

fn default_pokt_supported_chains() -> HashMap<String, String> {
    HashMap::from([
        // Solana Mainnet
        (
            "solana:4sgjmw1sunhzsxgspuhpqldx6wiyjntz".into(),
            "solana-mainnet".into(),
        ),
        // Avax C-Chain
        ("eip155:43114".into(), "avax-mainnet".into()),
        // Gnosis
        ("eip155:100".into(), "poa-xdai".into()),
        // Binance Smart Chain
        // There's a dedicated BSC provider
        // ("eip155:56".into(), "bsc-mainnet".into()),
    ])
}
