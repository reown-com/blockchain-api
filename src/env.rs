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
    #[serde(default = "default_infura_supported_chains")]
    pub infura_supported_chains: HashMap<String, String>,
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
        ("eip155:1".into(), "mainnet".into()),
        ("eip155:3".into(), "ropsten".into()),
        ("eip155:42".into(), "kovan".into()),
        ("eip155:4".into(), "rinkeby".into()),
        ("eip155:5".into(), "goerli".into()),
    ])
}

pub fn get_config() -> error::Result<Config> {
    let config = envy::from_env::<Config>()?;
    Ok(config)
}
