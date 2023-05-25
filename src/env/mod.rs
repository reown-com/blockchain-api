use {
    crate::{
        analytics::Config as AnalyticsConfig,
        error,
        project::{storage::Config as StorageConfig, Config as RegistryConfig},
        providers::{ProviderKind, Weight},
    },
    serde::de::DeserializeOwned,
    std::collections::HashMap,
};

mod binance;
mod infura;
mod omnia;
mod pokt;
mod publicnode;
mod server;
mod zksync;

pub use {binance::*, infura::*, omnia::*, pokt::*, publicnode::*, server::*, zksync::*};

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub registry: RegistryConfig,
    pub storage: StorageConfig,
    pub analytics: AnalyticsConfig,
}

impl Config {
    pub fn from_env() -> error::RpcResult<Config> {
        Ok(Self {
            server: from_env("RPC_PROXY_")?,
            registry: from_env("RPC_PROXY_REGISTRY_")?,
            storage: from_env("RPC_PROXY_STORAGE_")?,
            analytics: from_env("RPC_PROXY_ANALYTICS_")?,
        })
    }
}

fn from_env<T: DeserializeOwned>(prefix: &str) -> Result<T, envy::Error> {
    envy::prefixed(prefix).from_env()
}

// TODO: Is this required
pub trait ProviderConfig {
    fn supported_chains(&self) -> &HashMap<String, (String, Weight)>;
    fn provider_kind(&self) -> ProviderKind;
}
