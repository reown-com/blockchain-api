use std::sync::LazyLock;

use crate::providers::Priority;

pub static ACTIVE_CONFIG: LazyLock<Config> = LazyLock::new(|| Config {
    chains: vec![ChainConfig {
        caip2: "eip155:1".to_string(),
        name: "Ethereum Mainnet".to_string(),
        providers: vec![ProviderConfig {
            url: "https://eth.drpc.org".to_string(),
            priority: Priority::Minimal,
        }],
    }],
});

#[derive(Debug, Clone)]
pub struct Config {
    pub chains: Vec<ChainConfig>,
}

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub caip2: String,
    pub name: String,
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub url: String,
    pub priority: Priority,
}
