use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::error;
use crate::project::storage::Config as StorageConfig;
use crate::project::Config as RegistryConfig;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

// #############################################################################

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InfuraConfig {
    pub project_id: String,

    #[serde(default = "default_infura_supported_chains")]
    pub supported_chains: HashMap<String, String>,
}

// #############################################################################

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PoktConfig {
    pub project_id: String,

    #[serde(default = "default_pokt_supported_chains")]
    pub supported_chains: HashMap<String, String>,
}

// #############################################################################

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub infura: InfuraConfig,
    pub pokt: PoktConfig,
    pub registry: RegistryConfig,
    pub storage: StorageConfig,
}

impl Config {
    pub fn from_env() -> error::RpcResult<Config> {
        Ok(Self {
            server: from_env("RPC_PROXY_")?,
            infura: from_env("RPC_PROXY_INFURA_")?,
            pokt: from_env("RPC_PROXY_POKT_")?,
            registry: from_env("RPC_PROXY_REGISTRY_")?,
            storage: from_env("RPC_PROXY_STORAGE_")?,
        })
    }
}

fn from_env<T: DeserializeOwned>(prefix: &str) -> Result<T, envy::Error> {
    envy::prefixed(prefix).from_env()
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
        // Gnosis
        ("eip155:100".into(), "poa-xdai".into()),
    ])
}
