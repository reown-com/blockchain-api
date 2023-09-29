use {
    crate::{
        analytics::Config as AnalyticsConfig,
        error,
        profiler::ProfilerConfig,
        project::{storage::Config as StorageConfig, Config as RegistryConfig},
        providers::{ProviderKind, Weight},
    },
    serde::de::DeserializeOwned,
    std::{collections::HashMap, fmt::Display},
};

mod base;
mod binance;
mod infura;
mod omnia;
mod pokt;
mod publicnode;
mod server;
mod zksync;
mod zora;

pub use {
    base::*,
    binance::*,
    infura::*,
    omnia::*,
    pokt::*,
    publicnode::*,
    server::*,
    zksync::*,
    zora::*,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainId(pub String);

impl Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server: ServerConfig,
    pub registry: RegistryConfig,
    pub storage: StorageConfig,
    pub analytics: AnalyticsConfig,
    pub profiler: ProfilerConfig,
}

impl Config {
    pub fn from_env() -> error::RpcResult<Config> {
        Ok(Self {
            server: from_env("RPC_PROXY_")?,
            registry: from_env("RPC_PROXY_REGISTRY_")?,
            storage: from_env("RPC_PROXY_STORAGE_")?,
            analytics: from_env("RPC_PROXY_ANALYTICS_")?,
            profiler: from_env("RPC_PROXY_PROFILER_")?,
        })
    }
}

fn from_env<T: DeserializeOwned>(prefix: &str) -> Result<T, envy::Error> {
    envy::prefixed(prefix).from_env()
}

pub trait ProviderConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)>;
    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)>;
    fn provider_kind(&self) -> ProviderKind;
}

#[cfg(test)]
mod test {
    use {
        crate::{
            analytics,
            env::{Config, ServerConfig},
            profiler::ProfilerConfig,
            project,
        },
        std::net::Ipv4Addr,
    };

    #[test]
    fn ensure_env_var_config() {
        let values = [
            // Server config.
            ("RPC_PROXY_HOST", "1.2.3.4"),
            ("RPC_PROXY_PORT", "123"),
            ("RPC_PROXY_PRIVATE_PORT", "234"),
            ("RPC_PROXY_LOG_LEVEL", "TRACE"),
            ("RPC_PROXY_EXTERNAL_IP", "2.3.4.5"),
            ("RPC_PROXY_BLOCKED_COUNTRIES", "KP,IR,CU,SY"),
            // Registry config.
            ("RPC_PROXY_REGISTRY_API_URL", "API_URL"),
            ("RPC_PROXY_REGISTRY_API_AUTH_TOKEN", "API_AUTH_TOKEN"),
            ("RPC_PROXY_REGISTRY_PROJECT_DATA_CACHE_TTL", "345"),
            // Storage config.
            ("RPC_PROXY_STORAGE_REDIS_MAX_CONNECTIONS", "456"),
            (
                "RPC_PROXY_STORAGE_PROJECT_DATA_REDIS_ADDR_READ",
                "redis://127.0.0.1/data/read",
            ),
            (
                "RPC_PROXY_STORAGE_PROJECT_DATA_REDIS_ADDR_WRITE",
                "redis://127.0.0.1/data/write",
            ),
            (
                "RPC_PROXY_STORAGE_IDENTITY_CACHE_REDIS_ADDR_READ",
                "redis://127.0.0.1/identity/read",
            ),
            (
                "RPC_PROXY_STORAGE_IDENTITY_CACHE_REDIS_ADDR_WRITE",
                "redis://127.0.0.1/identity/write",
            ),
            // Analytics config.
            ("RPC_PROXY_ANALYTICS_S3_ENDPOINT", "s3://127.0.0.1"),
            ("RPC_PROXY_ANALYTICS_EXPORT_BUCKET", "EXPORT_BUCKET"),
            ("RPC_PROXY_ANALYTICS_GEOIP_DB_BUCKET", "GEOIP_DB_BUCKET"),
            ("RPC_PROXY_ANALYTICS_GEOIP_DB_KEY", "GEOIP_DB_KEY"),
        ];

        values.iter().for_each(set_env_var);

        assert_eq!(Config::from_env().unwrap(), Config {
            server: ServerConfig {
                host: "1.2.3.4".to_owned(),
                port: 123,
                private_port: 234,
                log_level: "TRACE".to_owned(),
                external_ip: Some(Ipv4Addr::new(2, 3, 4, 5).into()),
                blocked_countries: vec![
                    "KP".to_owned(),
                    "IR".to_owned(),
                    "CU".to_owned(),
                    "SY".to_owned()
                ],
            },
            registry: project::Config {
                api_url: Some("API_URL".to_owned()),
                api_auth_token: Some("API_AUTH_TOKEN".to_owned()),
                project_data_cache_ttl: 345,
            },
            storage: project::storage::Config {
                redis_max_connections: 456,
                project_data_redis_addr_read: Some("redis://127.0.0.1/data/read".to_owned()),
                project_data_redis_addr_write: Some("redis://127.0.0.1/data/write".to_owned()),
                identity_cache_redis_addr_read: Some("redis://127.0.0.1/identity/read".to_owned()),
                identity_cache_redis_addr_write: Some(
                    "redis://127.0.0.1/identity/write".to_owned()
                ),
            },
            analytics: analytics::Config {
                s3_endpoint: Some("s3://127.0.0.1".to_owned()),
                export_bucket: Some("EXPORT_BUCKET".to_owned()),
                geoip_db_bucket: Some("GEOIP_DB_BUCKET".to_owned()),
                geoip_db_key: Some("GEOIP_DB_KEY".to_owned()),
            },
            profiler: ProfilerConfig {},
        });

        values.iter().for_each(reset_env_var);
    }

    fn set_env_var((key, value): &(&str, &str)) {
        std::env::set_var(key, value)
    }

    fn reset_env_var((key, ..): &(&str, &str)) {
        std::env::remove_var(key)
    }
}
