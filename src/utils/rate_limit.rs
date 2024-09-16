use {
    crate::metrics::Metrics,
    chrono::{Duration, Utc},
    deadpool_redis::Pool,
    moka::future::Cache,
    serde::Deserialize,
    std::{sync::Arc, time::SystemTime},
    tracing::error,
    wc::rate_limit::{token_bucket, RateLimitError, RateLimitExceeded},
};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct RateLimitingConfig {
    pub max_tokens: Option<u32>,
    pub refill_interval_sec: Option<u32>,
    pub refill_rate: Option<u32>,
    pub ip_white_list: Option<Vec<String>>,
}

pub struct RateLimit {
    mem_cache: Cache<String, u64>,
    redis_pool: Arc<Pool>,
    max_tokens: u32,
    interval: Duration,
    refill_rate: u32,
    metrics: Arc<Metrics>,
    ip_white_list: Option<Vec<String>>,
}

impl RateLimit {
    pub fn new(
        redis_addr: &str,
        redis_pool_max_size: usize,
        max_tokens: u32,
        interval: Duration,
        refill_rate: u32,
        metrics: Arc<Metrics>,
        ip_white_list: Option<Vec<String>>,
    ) -> Option<Self> {
        let redis_builder = deadpool_redis::Config::from_url(redis_addr)
            .builder()
            .map_err(|e| {
                error!(
                    "Failed to create redis pool builder for rate limiting: {:?}",
                    e
                );
            })
            .ok()?
            .max_size(redis_pool_max_size)
            .runtime(deadpool_redis::Runtime::Tokio1)
            .build();

        let redis_pool = match redis_builder {
            Ok(pool) => Arc::new(pool),
            Err(e) => {
                error!("Failed to create redis pool for rate limiting: {:?}", e);
                return None;
            }
        };
        let mem_cache = Cache::builder()
            .time_to_live(
                interval
                    .to_std()
                    .expect("Failed to convert duration for rate limiting memory cache"),
            )
            .build();
        Some(Self {
            mem_cache,
            redis_pool,
            max_tokens,
            interval,
            refill_rate,
            metrics,
            ip_white_list,
        })
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
        // Check first if the IP is in the white list
        if let Some(white_list) = &self.ip_white_list {
            if white_list.contains(&ip.to_string()) {
                return Ok(());
            }
        }

        let call_start_time = SystemTime::now();
        let result = token_bucket(
            &self.mem_cache.clone(),
            &self.redis_pool.clone(),
            self.format_key(endpoint, ip),
            self.max_tokens,
            self.interval,
            self.refill_rate,
            Utc::now(),
        )
        .await;
        self.metrics.add_rate_limiting_latency(call_start_time);

        match result {
            Ok(_) => Ok(()),
            Err(e) => match e {
                RateLimitError::RateLimitExceeded(e) => {
                    self.metrics.add_rate_limited_response();
                    Err(e)
                }
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
