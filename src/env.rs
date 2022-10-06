use crate::error;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub infura_project_id: String,
    pub pokt_project_id: String,
    #[serde(default = "default_infura_supported_chains")]
    pub infura_supported_chains: HashMap<String, String>,
    #[serde(default = "default_pokt_supported_chains")]
    pub pokt_supported_chains: HashMap<String, String>,
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_log_level() -> String {
    "WARN".to_string()
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
    ])
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
    ])
}

pub fn get_config() -> error::RpcResult<Config> {
    let config = envy::from_env::<Config>()?;
    Ok(config)
}
