use {
    chrono::{Duration, Utc},
    deadpool_redis::Pool,
    moka::future::Cache,
    std::sync::Arc,
    tracing::error,
    wc::rate_limit::{token_bucket, RateLimitError, RateLimitExceeded},
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

    /// Checks if the given endpoint, ip and project ID is rate limited
    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn is_rate_limited(
        &self,
        endpoint: &str,
        ip: &str,
        _project_id: Option<&str>,
    ) -> Result<(), RateLimitExceeded> {
        match token_bucket(
            &self.mem_cache.clone(),
            &self.redis_pool.clone(),
            self.format_key(endpoint, ip),
            self.max_tokens,
            self.interval,
            self.refill_rate,
            Utc::now(),
        )
        .await
        {
            Ok(_) => Ok(()),
            Err(e) => match e {
                RateLimitError::RateLimitExceeded(e) => Err(e),
                RateLimitError::Internal(e) => {
                    error!("Internal rate limiting error: {:?}", e);
                    Ok(())
                }
            },
        }
    }

    /// Returns the current rate limited entries count
    pub async fn get_rate_limited_count(&self) -> u64 {
        self.mem_cache.run_pending_tasks().await;
        self.mem_cache.entry_count()
    }
}
