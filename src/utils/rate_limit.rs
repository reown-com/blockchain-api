use {
    chrono::{Duration, Utc},
    deadpool_redis::Pool,
    moka::future::Cache,
    std::sync::Arc,
    wc::rate_limit::{token_bucket, RateLimitError},
};

pub struct RateLimit {
    mem_cache: Cache<String, u64>,
    redis_pool: Arc<Pool>,
    max_tokens: u32,
    interval: Duration,
    refill_rate: u32,
}

impl RateLimit {
    pub fn new(redis_addr: &str, max_tokens: u32, interval: Duration, refill_rate: u32) -> Self {
        let redis_pool = Arc::new(
            deadpool_redis::Config::from_url(redis_addr)
                .create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .expect("Failed to create redis pool for rate limiting"),
        );
        let mem_cache = Cache::builder()
            .time_to_live(
                interval
                    .to_std()
                    .expect("Failed to convert duration for rate limiting memory cache"),
            )
            .build();
        Self {
            mem_cache,
            redis_pool,
            max_tokens,
            interval,
            refill_rate,
        }
    }

    fn format_key(&self, endpoint: &str, ip: &str) -> String {
        format!("rate_limit:{}:{}", endpoint, ip)
    }

    pub async fn is_rate_limited(
        &self,
        endpoint: &str,
        ip: &str,
        _project_id: Option<&str>,
    ) -> Result<(), RateLimitError> {
        tracing::info!(
            "Rate limiting check for endpoint: {} and ip: {}",
            endpoint,
            ip
        );
        // TODO:
        // * Handle properly when the redis pool is not available (429 always)
        // * Add metrcis for rate limiting
        // * Add analytics for rate limiting
        token_bucket(
            &self.mem_cache.clone(),
            &self.redis_pool.clone(),
            self.format_key(endpoint, ip),
            self.max_tokens,
            self.interval,
            self.refill_rate,
            Utc::now(),
        )
        .await
    }
}
