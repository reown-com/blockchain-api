use {
    crate::{
        analytics::Config as AnalyticsConfig,
        database::config::PostgresConfig,
        error,
        profiler::ProfilerConfig,
        project::{storage::Config as StorageConfig, Config as RegistryConfig},
        providers::{ProviderKind, ProvidersConfig, Weight},
        utils::rate_limit::RateLimitingConfig,
    },
    serde::de::DeserializeOwned,
    std::{collections::HashMap, fmt::Display},
};
pub use {
    aurora::*,
    base::*,
    binance::*,
    getblock::*,
    infura::*,
    mantle::*,
    near::*,
    pokt::*,
    publicnode::*,
    quicknode::*,
    server::*,
    zksync::*,
    zora::*,
};
mod aurora;
mod base;
mod binance;
mod getblock;
mod infura;
mod mantle;
mod near;
mod pokt;
mod publicnode;
mod quicknode;
mod server;
mod zksync;
mod zora;

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
    pub postgres: PostgresConfig,
    pub analytics: AnalyticsConfig,
    pub profiler: ProfilerConfig,
    pub providers: ProvidersConfig,
    pub rate_limiting: RateLimitingConfig,
}

impl Config {
    pub fn from_env() -> error::RpcResult<Config> {
        Ok(Self {
            server: from_env("RPC_PROXY_")?,
            registry: from_env("RPC_PROXY_REGISTRY_")?,
            storage: from_env("RPC_PROXY_STORAGE_")?,
            postgres: from_env("RPC_PROXY_POSTGRES_")?,
            analytics: from_env("RPC_PROXY_ANALYTICS_")?,
            profiler: from_env("RPC_PROXY_PROFILER_")?,
            providers: from_env("RPC_PROXY_PROVIDER_")?,
            rate_limiting: from_env("RPC_PROXY_RATE_LIMITING_")?,
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
            database::config::PostgresConfig,
            env::{Config, ServerConfig},
            profiler::ProfilerConfig,
            project,
            providers::ProvidersConfig,
            utils::rate_limit::RateLimitingConfig,
        },
        std::net::Ipv4Addr,
    };

    #[test]
    fn ensure_env_var_config() {
        let values = [
            // Server config.
            ("RPC_PROXY_HOST", "1.2.3.4"),
            ("RPC_PROXY_PORT", "123"),
            ("RPC_PROXY_PROMETHEUS_PORT", "234"),
            ("RPC_PROXY_LOG_LEVEL", "TRACE"),
            ("RPC_PROXY_EXTERNAL_IP", "2.3.4.5"),
            ("RPC_PROXY_BLOCKED_COUNTRIES", "KP,IR,CU,SY"),
            ("RPC_PROXY_GEOIP_DB_BUCKET", "GEOIP_DB_BUCKET"),
            ("RPC_PROXY_GEOIP_DB_KEY", "GEOIP_DB_KEY"),
            // Integration tests config.
            ("RPC_PROXY_TESTING_PROJECT_ID", "TESTING_PROJECT_ID"),
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
            (
                "RPC_PROXY_STORAGE_RATE_LIMITING_CACHE_REDIS_ADDR_READ",
                "redis://127.0.0.1/rate_limit/read",
            ),
            (
                "RPC_PROXY_STORAGE_RATE_LIMITING_CACHE_REDIS_ADDR_WRITE",
                "redis://127.0.0.1/rate_limit/write",
            ),
            // Analytics config.
            ("RPC_PROXY_ANALYTICS_S3_ENDPOINT", "s3://127.0.0.1"),
            ("RPC_PROXY_ANALYTICS_EXPORT_BUCKET", "EXPORT_BUCKET"),
            // Providers config
            ("RPC_PROXY_PROVIDER_INFURA_PROJECT_ID", "INFURA_PROJECT_ID"),
            ("RPC_PROXY_PROVIDER_POKT_PROJECT_ID", "POKT_PROJECT_ID"),
            ("RPC_PROXY_PROVIDER_ZERION_API_KEY", "ZERION_API_KEY"),
            (
                "RPC_PROXY_PROVIDER_QUICKNODE_API_TOKEN",
                "QUICKNODE_API_TOKEN",
            ),
            ("RPC_PROXY_PROVIDER_COINBASE_API_KEY", "COINBASE_API_KEY"),
            ("RPC_PROXY_PROVIDER_COINBASE_APP_ID", "COINBASE_APP_ID"),
            ("RPC_PROXY_PROVIDER_ONE_INCH_API_KEY", "ONE_INCH_API_KEY"),
            ("RPC_PROXY_PROVIDER_GETBLOCK_ACCESS_TOKENS", "{}"),
            (
                "RPC_PROXY_PROVIDER_PROMETHEUS_QUERY_URL",
                "PROMETHEUS_QUERY_URL",
            ),
            (
                "RPC_PROXY_PROVIDER_PROMETHEUS_WORKSPACE_HEADER",
                "PROMETHEUS_WORKSPACE_HEADER",
            ),
            // Postgres config.
            (
                "RPC_PROXY_POSTGRES_URI",
                "postgres://postgres@localhost:5432/postgres",
            ),
            ("RPC_PROXY_POSTGRES_MAX_CONNECTIONS", "32"),
            // Rate limiting config.
            ("RPC_PROXY_RATE_LIMITING_MAX_TOKENS", "100"),
            ("RPC_PROXY_RATE_LIMITING_REFILL_INTERVAL_SEC", "1"),
            ("RPC_PROXY_RATE_LIMITING_REFILL_RATE", "10"),
        ];

        values.iter().for_each(set_env_var);

        assert_eq!(Config::from_env().unwrap(), Config {
            server: ServerConfig {
                host: "1.2.3.4".to_owned(),
                port: 123,
                prometheus_port: 234,
                log_level: "TRACE".to_owned(),
                external_ip: Some(Ipv4Addr::new(2, 3, 4, 5).into()),
                blocked_countries: vec![
                    "KP".to_owned(),
                    "IR".to_owned(),
                    "CU".to_owned(),
                    "SY".to_owned(),
                ],
                s3_endpoint: None,
                geoip_db_bucket: Some("GEOIP_DB_BUCKET".to_owned()),
                geoip_db_key: Some("GEOIP_DB_KEY".to_owned()),
                testing_project_id: Some("TESTING_PROJECT_ID".to_owned()),
                validate_project_id: true,
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
                rate_limiting_cache_redis_addr_read: Some(
                    "redis://127.0.0.1/rate_limit/read".to_owned()
                ),
                rate_limiting_cache_redis_addr_write: Some(
                    "redis://127.0.0.1/rate_limit/write".to_owned()
                ),
            },
            postgres: PostgresConfig {
                uri: "postgres://postgres@localhost:5432/postgres".to_owned(),
                max_connections: 32,
            },
            analytics: analytics::Config {
                s3_endpoint: Some("s3://127.0.0.1".to_owned()),
                export_bucket: Some("EXPORT_BUCKET".to_owned()),
            },
            profiler: ProfilerConfig {},
            providers: ProvidersConfig {
                prometheus_query_url: Some("PROMETHEUS_QUERY_URL".to_owned()),
                prometheus_workspace_header: Some("PROMETHEUS_WORKSPACE_HEADER".to_owned()),
                infura_project_id: "INFURA_PROJECT_ID".to_string(),
                pokt_project_id: "POKT_PROJECT_ID".to_string(),
                quicknode_api_token: "QUICKNODE_API_TOKEN".to_string(),
                zerion_api_key: Some("ZERION_API_KEY".to_owned()),
                coinbase_api_key: Some("COINBASE_API_KEY".to_owned()),
                coinbase_app_id: Some("COINBASE_APP_ID".to_owned()),
                one_inch_api_key: Some("ONE_INCH_API_KEY".to_owned()),
                getblock_access_tokens: Some("{}".to_owned()),
            },
            rate_limiting: RateLimitingConfig {
                max_tokens: Some(100),
                refill_interval_sec: Some(1),
                refill_rate: Some(10),
            },
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
