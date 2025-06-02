use crate::config::{ChainConfig, Config, ProviderConfig};
use std::sync::LazyLock;

pub static ACTIVE_CONFIG: LazyLock<Config> = LazyLock::new(|| Config {
    chains: vec![ChainConfig {
        caip2: "eip155:1".to_string(),
        name: "Ethereum Mainnet".to_string(),
        providers: vec![ProviderConfig {
            url: "https://eth.drpc.org".to_string(),
        }],
    }],
});
