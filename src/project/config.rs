use {
    serde::Deserialize, serde_piecewise_default::DeserializePiecewiseDefault, std::time::Duration,
};

#[derive(DeserializePiecewiseDefault, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub api_url: Option<String>,
    pub api_auth_token: Option<String>,
    pub project_data_cache_ttl: u64,
    pub circuit_cooldown_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: None,
            api_auth_token: None,
            project_data_cache_ttl: 60 * 5,
            circuit_cooldown_ms: 1_000,
        }
    }
}

impl Config {
    pub fn project_data_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.project_data_cache_ttl)
    }

    pub fn circuit_cooldown(&self) -> Duration {
        Duration::from_millis(self.circuit_cooldown_ms)
    }
}
