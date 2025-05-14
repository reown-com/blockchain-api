use {
    crate::{
        analytics::Config as AnalyticsConfig,
        database::config::PostgresConfig,
        error,
        handlers::balance::Config as BalanceConfig,
        handlers::wallet::exchanges::Config as ExchangesConfig,
        names::Config as NamesConfig,
        profiler::ProfilerConfig,
        project::{storage::Config as StorageConfig, Config as RegistryConfig},
        providers::{ProviderKind, ProvidersConfig, Weight},
        storage::irn::Config as IrnConfig,
        utils::{crypto::CaipNamespaces, rate_limit::RateLimitingConfig},
    },
    serde::de::DeserializeOwned,
    std::{collections::HashMap, fmt::Display},
};
pub use {
    allnodes::*, arbitrum::*, aurora::*, base::*, binance::*, drpc::*, dune::*, edexa::*, getblock::*,
    infura::*, mantle::*, monad::*, morph::*, near::*, odyssey::*, pokt::*, publicnode::*,
    quicknode::*, server::*, solscan::*, syndica::*, unichain::*, wemix::*, zerion::*, zksync::*,
    zora::*,
};
mod allnodes;
mod arbitrum;
mod aurora;
mod base;
mod binance;
mod drpc;
mod dune;
mod edexa;
mod getblock;
mod infura;
mod mantle;
mod monad;
mod morph;
mod near;
mod odyssey;
mod pokt;
mod publicnode;
mod quicknode;
mod server;
pub mod solscan;
mod syndica;
mod unichain;
mod wemix;
pub mod zerion;
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
    pub irn: IrnConfig,
    pub names: NamesConfig,
    pub balances: BalanceConfig,
    pub exchanges: ExchangesConfig,
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
            irn: from_env("RPC_PROXY_IRN_")?,
            names: from_env("RPC_PROXY_NAMES_")?,
            balances: from_env("RPC_PROXY_BALANCES_")?,
            exchanges: from_env("RPC_PROXY_EXCHANGES_")?,
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

pub trait BalanceProviderConfig {
    fn supported_namespaces(self) -> HashMap<CaipNamespaces, Weight>;
    fn provider_kind(&self) -> ProviderKind;
}

#[cfg(test)]
#[cfg(not(feature = "test-mock-bundler"))] // These tests depend on environment variables
mod test {
    use {
        crate::{
            analytics,
            database::config::PostgresConfig,
            env::{Config, ServerConfig},
            handlers::balance::Config as BalanceConfig,
            handlers::wallet::exchanges::Config as ExchangesConfig,
            names::Config as NamesConfig,
            profiler::ProfilerConfig,
            project,
            providers::ProvidersConfig,
            storage::irn::Config as IrnConfig,
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
            (
                "RPC_PROXY_PROVIDER_CACHE_REDIS_ADDR",
                "redis://127.0.0.1/providers_cache",
            ),
            ("RPC_PROXY_PROVIDER_INFURA_PROJECT_ID", "INFURA_PROJECT_ID"),
            ("RPC_PROXY_PROVIDER_POKT_PROJECT_ID", "POKT_PROJECT_ID"),
            ("RPC_PROXY_PROVIDER_ZERION_API_KEY", "ZERION_API_KEY"),
            (
                "RPC_PROXY_PROVIDER_QUICKNODE_API_TOKENS",
                "QUICKNODE_API_TOKENS",
            ),
            ("RPC_PROXY_PROVIDER_COINBASE_API_KEY", "COINBASE_API_KEY"),
            ("RPC_PROXY_PROVIDER_COINBASE_APP_ID", "COINBASE_APP_ID"),
            ("RPC_PROXY_PROVIDER_ONE_INCH_API_KEY", "ONE_INCH_API_KEY"),
            ("RPC_PROXY_PROVIDER_ONE_INCH_REFERRER", "ONE_INCH_REFERRER"),
            ("RPC_PROXY_PROVIDER_GETBLOCK_ACCESS_TOKENS", "{}"),
            ("RPC_PROXY_PROVIDER_PIMLICO_API_KEY", "PIMLICO_API_KEY"),
            (
                "RPC_PROXY_PROVIDER_SOLSCAN_API_V2_TOKEN",
                "SOLSCAN_API_V2_TOKEN",
            ),
            ("RPC_PROXY_PROVIDER_BUNGEE_API_KEY", "BUNGEE_API_KEY"),
            ("RPC_PROXY_PROVIDER_TENDERLY_API_KEY", "TENDERLY_KEY"),
            (
                "RPC_PROXY_PROVIDER_TENDERLY_ACCOUNT_ID",
                "TENDERLY_ACCOUNT_ID",
            ),
            (
                "RPC_PROXY_PROVIDER_TENDERLY_PROJECT_ID",
                "TENDERLY_PROJECT_ID",
            ),
            ("RPC_PROXY_PROVIDER_DUNE_API_KEY", "DUNE_API_KEY"),
            ("RPC_PROXY_PROVIDER_SYNDICA_API_KEY", "SYNDICA_API_KEY"),
            ("RPC_PROXY_PROVIDER_ALLNODES_API_KEY", "ALLNODES_API_KEY"),
            ("RPC_PROXY_PROVIDER_MELD_API_KEY", "MELD_API_KEY"),
            ("RPC_PROXY_PROVIDER_MELD_API_URL", "MELD_API_URL"),
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
            (
                "RPC_PROXY_RATE_LIMITING_IP_WHITELIST",
                "127.0.0.1,127.0.0.2",
            ),
            // IRN config.
            ("RPC_PROXY_IRN_NODES", "node1.id,node2.id"),
            ("RPC_PROXY_IRN_KEY", "key"),
            ("RPC_PROXY_IRN_NAMESPACE", "namespace"),
            ("RPC_PROXY_IRN_NAMESPACE_SECRET", "namespace"),
            // Names configuration
            ("RPC_PROXY_NAMES_ALLOWED_ZONES", "test1.id,test2.id"),
            // Account balances-related configuration
            ("RPC_PROXY_BALANCES_DENYLIST_PROJECT_IDS", "test_project_id"),
            // Exchanges configuration
            (
                "RPC_PROXY_EXCHANGES_COINBASE_PROJECT_ID",
                "COINBASE_PROJECT_ID",
            ),
            ("RPC_PROXY_EXCHANGES_COINBASE_KEY_NAME", "COINBASE_KEY_NAME"),
            (
                "RPC_PROXY_EXCHANGES_COINBASE_KEY_SECRET",
                "COINBASE_KEY_SECRET",
            ),
            ("RPC_PROXY_EXCHANGES_BINANCE_CLIENT_ID", "BINANCE_CLIENT_ID"),
            ("RPC_PROXY_EXCHANGES_BINANCE_TOKEN", "BINANCE_TOKEN"),
            ("RPC_PROXY_EXCHANGES_BINANCE_KEY", "BINANCE_KEY"),
            ("RPC_PROXY_EXCHANGES_BINANCE_HOST", "BINANCE_HOST"),
        ];

        values.iter().for_each(set_env_var);

        assert_eq!(
            Config::from_env().unwrap(),
            Config {
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
                    identity_cache_redis_addr_read: Some(
                        "redis://127.0.0.1/identity/read".to_owned()
                    ),
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
                    cache_redis_addr: Some("redis://127.0.0.1/providers_cache".to_owned()),
                    infura_project_id: "INFURA_PROJECT_ID".to_string(),
                    pokt_project_id: "POKT_PROJECT_ID".to_string(),
                    quicknode_api_tokens: "QUICKNODE_API_TOKENS".to_string(),
                    zerion_api_key: "ZERION_API_KEY".to_owned(),
                    coinbase_api_key: Some("COINBASE_API_KEY".to_owned()),
                    coinbase_app_id: Some("COINBASE_APP_ID".to_owned()),
                    one_inch_api_key: Some("ONE_INCH_API_KEY".to_owned()),
                    one_inch_referrer: Some("ONE_INCH_REFERRER".to_owned()),
                    getblock_access_tokens: Some("{}".to_owned()),
                    pimlico_api_key: "PIMLICO_API_KEY".to_string(),
                    solscan_api_v2_token: "SOLSCAN_API_V2_TOKEN".to_string(),
                    bungee_api_key: "BUNGEE_API_KEY".to_string(),
                    tenderly_api_key: "TENDERLY_KEY".to_string(),
                    tenderly_account_id: "TENDERLY_ACCOUNT_ID".to_string(),
                    tenderly_project_id: "TENDERLY_PROJECT_ID".to_string(),
                    dune_api_key: "DUNE_API_KEY".to_string(),
                    syndica_api_key: "SYNDICA_API_KEY".to_string(),
                    override_bundler_urls: None,
                    allnodes_api_key: "ALLNODES_API_KEY".to_string(),
                    meld_api_key: "MELD_API_KEY".to_string(),
                    meld_api_url: "MELD_API_URL".to_string(),
                },
                rate_limiting: RateLimitingConfig {
                    max_tokens: Some(100),
                    refill_interval_sec: Some(1),
                    refill_rate: Some(10),
                    ip_whitelist: Some(vec!["127.0.0.1".into(), "127.0.0.2".into()]),
                },
                irn: IrnConfig {
                    nodes: Some(vec!["node1.id".to_owned(), "node2.id".to_owned()]),
                    key: Some("key".to_owned()),
                    namespace: Some("namespace".to_owned()),
                    namespace_secret: Some("namespace".to_owned()),
                },
                names: NamesConfig {
                    allowed_zones: Some(vec!["test1.id".to_owned(), "test2.id".to_owned()]),
                },
                balances: BalanceConfig {
                    denylist_project_ids: Some(vec!["test_project_id".to_owned()]),
                },
                exchanges: ExchangesConfig {
                    coinbase_project_id: Some("COINBASE_PROJECT_ID".to_owned()),
                    binance_client_id: Some("BINANCE_CLIENT_ID".to_owned()),
                    binance_token: Some("BINANCE_TOKEN".to_owned()),
                    binance_key: Some("BINANCE_KEY".to_owned()),
                    binance_host: Some("BINANCE_HOST".to_owned()),
                    coinbase_key_name: Some("COINBASE_KEY_NAME".to_owned()),
                    coinbase_key_secret: Some("COINBASE_KEY_SECRET".to_owned()),
                },
            }
        );

        values.iter().for_each(reset_env_var);
    }

    fn set_env_var((key, value): &(&str, &str)) {
        std::env::set_var(key, value)
    }

    fn reset_env_var((key, ..): &(&str, &str)) {
        std::env::remove_var(key)
    }
}
