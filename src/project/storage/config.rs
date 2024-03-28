use {
    crate::storage::redis::Addr as RedisAddr,
    serde::Deserialize,
    serde_piecewise_default::DeserializePiecewiseDefault,
};

#[derive(DeserializePiecewiseDefault, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub redis_max_connections: usize,
    pub project_data_redis_addr_read: Option<String>,
    pub project_data_redis_addr_write: Option<String>,
    pub identity_cache_redis_addr_read: Option<String>,
    pub identity_cache_redis_addr_write: Option<String>,
    pub rate_limiting_cache_redis_addr_read: Option<String>,
    pub rate_limiting_cache_redis_addr_write: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            redis_max_connections: 64,
            project_data_redis_addr_read: None,
            project_data_redis_addr_write: None,
            identity_cache_redis_addr_read: None,
            identity_cache_redis_addr_write: None,
            rate_limiting_cache_redis_addr_read: None,
            rate_limiting_cache_redis_addr_write: None,
        }
    }
}

impl Config {
    pub fn project_data_redis_addr(&self) -> Option<RedisAddr> {
        match (
            &self.project_data_redis_addr_read,
            &self.project_data_redis_addr_write,
        ) {
            (None, None) => None,
            (addr_read, addr_write) => Some(RedisAddr::from((addr_read, addr_write))),
        }
    }

    pub fn identity_cache_redis_addr(&self) -> Option<RedisAddr> {
        match (
            &self.identity_cache_redis_addr_read,
            &self.identity_cache_redis_addr_write,
        ) {
            (None, None) => None,
            (addr_read, addr_write) => Some(RedisAddr::from((addr_read, addr_write))),
        }
    }

    pub fn rate_limiting_cache_redis_addr(&self) -> Option<RedisAddr> {
        match (
            &self.rate_limiting_cache_redis_addr_read,
            &self.rate_limiting_cache_redis_addr_write,
        ) {
            (None, None) => None,
            (addr_read, addr_write) => Some(RedisAddr::from((addr_read, addr_write))),
        }
    }
}
