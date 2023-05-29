use {
    crate::{
        analytics::Config as AnalyticsConfig,
        error,
        project::{storage::Config as StorageConfig, Config as RegistryConfig},
        providers::{ProviderKind, Weight},
    },
    serde::de::DeserializeOwned,
    std::{collections::HashMap, fmt::Display},
};

mod binance;
mod infura;
mod omnia;
mod pokt;
mod publicnode;
mod server;
mod zksync;

pub use {binance::*, infura::*, omnia::*, pokt::*, publicnode::*, server::*, zksync::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainId(pub String);

impl Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub registry: RegistryConfig,
    pub storage: StorageConfig,
    pub analytics: AnalyticsConfig,
    pub prometheus_query_url: String,
}

impl Config {
    pub fn from_env() -> error::RpcResult<Config> {
        Ok(Self {
            server: from_env("RPC_PROXY_")?,
            registry: from_env("RPC_PROXY_REGISTRY_")?,
            storage: from_env("RPC_PROXY_STORAGE_")?,
            analytics: from_env("RPC_PROXY_ANALYTICS_")?,
            prometheus_query_url: std::env::var("PROMETHEUS_QUERY_URL")
                .unwrap_or("http://localhost:9090".into()),
        })
    }
}

fn from_env<T: DeserializeOwned>(prefix: &str) -> Result<T, envy::Error> {
    envy::prefixed(prefix).from_env()
}

// TODO: Is this required
pub trait ProviderConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)>;
    fn provider_kind(&self) -> ProviderKind;
}
