use {
    crate::{
        analytics::Config as AnalyticsConfig,
        debug::DebugConfig,
        error,
        project::{storage::Config as StorageConfig, Config as RegistryConfig},
        providers::{ProviderKind, Weight},
    },
    serde::de::DeserializeOwned,
    std::{collections::HashMap, fmt::Display},
};

pub mod binance;
pub mod infura;
pub mod omnia;
pub mod pokt;
pub mod publicnode;
mod server;
pub mod zksync;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainId(pub String);

impl Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server: server::ServerConfig,
    pub registry: RegistryConfig,
    pub storage: StorageConfig,
    pub analytics: AnalyticsConfig,
    pub debug: DebugConfig,
}

impl Config {
    pub fn from_env() -> error::RpcResult<Config> {
        Ok(Self {
            server: from_env("RPC_PROXY_")?,
            registry: from_env("RPC_PROXY_REGISTRY_")?,
            storage: from_env("RPC_PROXY_STORAGE_")?,
            analytics: from_env("RPC_PROXY_ANALYTICS_")?,
            debug: from_env("RPC_PROXY_DEBUG_")?,
        })
    }
}

fn from_env<T: DeserializeOwned>(prefix: &str) -> Result<T, envy::Error> {
    envy::prefixed(prefix).from_env()
}

pub trait ProviderConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)>;
    fn provider_kind(&self) -> ProviderKind;
}
