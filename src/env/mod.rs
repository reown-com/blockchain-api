use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::analytics::Config as AnalyticsConfig;
use crate::error;
use crate::project::storage::Config as StorageConfig;
use crate::project::Config as RegistryConfig;

mod binance;
mod infura;
mod pokt;
mod server;
mod zksync;

pub use binance::*;
pub use infura::*;
pub use pokt::*;
pub use server::*;
pub use zksync::*;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub infura: InfuraConfig,
    pub pokt: PoktConfig,
    pub zksync: ZKSyncConfig,
    pub registry: RegistryConfig,
    pub storage: StorageConfig,
    pub analytics: AnalyticsConfig,
}

impl Config {
    pub fn from_env() -> error::RpcResult<Config> {
        Ok(Self {
            server: from_env("RPC_PROXY_")?,
            infura: from_env("RPC_PROXY_INFURA_")?,
            pokt: from_env("RPC_PROXY_POKT_")?,
            zksync: from_env("RPC_PROXY_ZKSYNC_")?,
            registry: from_env("RPC_PROXY_REGISTRY_")?,
            storage: from_env("RPC_PROXY_STORAGE_")?,
            analytics: from_env("RPC_PROXY_ANALYTICS_")?,
        })
    }
}

fn from_env<T: DeserializeOwned>(prefix: &str) -> Result<T, envy::Error> {
    envy::prefixed(prefix).from_env()
}
