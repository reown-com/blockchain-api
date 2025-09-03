use {
    crate::{
        error::{RpcError, RpcResult},
        project::{
            metrics::ProjectDataMetrics,
            storage::{Config as StorageConfig, ProjectDataResult, ProjectStorage},
        },
        storage::{error::StorageError, redis},
    },
    cerberus::{
        project::{
            PlanLimits, ProjectData, ProjectDataRequest, ProjectDataResponse,
            ProjectDataWithLimits, ProjectKey,
        },
        registry::{RegistryClient, RegistryError, RegistryHttpClient, RegistryResult},
    },
    std::{
        sync::atomic::{AtomicU64, Ordering},
        sync::Arc,
        time::{Duration, Instant},
    },
    tracing::error,
    wc::metrics::ServiceMetrics,
};
pub use {config::*, error::*};

mod config;
mod error;

pub mod metrics;
pub mod storage;

/// Circuit breaker cooldown period in milliseconds in case of registry internal error
/// to prevent the registry from being overwhelmed since we don't cache errors
const CIRCUIT_COOLDOWN_MS: u64 = 1_000;

#[derive(Debug, Clone)]
pub struct Registry {
    client: Option<RegistryHttpClient>,
    cache: Option<ProjectStorage>,
    metrics: ProjectDataMetrics,
    circuit_base_instant: Instant,
    circuit_last_error_ms: Arc<AtomicU64>,
    circuit_cooldown: Duration,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ResponseSource {
    Cache,
    Registry,
}

impl Registry {
    pub fn new(cfg_registry: &Config, cfg_storage: &StorageConfig) -> RpcResult<Self> {
        let meter = ServiceMetrics::meter();
        let metrics = ProjectDataMetrics::new(meter);

        let api_url = cfg_registry.api_url.as_ref();
        let api_auth_token = cfg_registry.api_auth_token.as_ref();

        let (client, cache) = if let Some(api_url) = api_url {
            let Some(api_auth_token) = api_auth_token else {
                return Err(RpcError::InvalidConfiguration(
                    "missing registry api_auth_token".to_string(),
                ));
            };

            let client = RegistryHttpClient::new(
                api_url,
                api_auth_token,
                "https://rpc-service.walletconnect.org",
                "blockchain-api",
                "1.0.0",
            )?;

            let cache_addr = cfg_storage.project_data_redis_addr();
            let cache = if let Some(cache_addr) = cache_addr {
                let cache = open_redis(&cache_addr, cfg_storage.redis_max_connections)?;

                Some(ProjectStorage::new(
                    cache,
                    cfg_registry.project_data_cache_ttl(),
                    metrics.clone(),
                ))
            } else {
                None
            };

            (Some(client), cache)
        } else {
            (None, None)
        };

        Ok(Self {
            client,
            cache,
            metrics,
            circuit_base_instant: Instant::now(),
            circuit_last_error_ms: Arc::new(AtomicU64::new(0)),
            circuit_cooldown: Duration::from_millis(CIRCUIT_COOLDOWN_MS),
        })
    }

    #[inline]
    fn now_ms_since_base(&self) -> u64 {
        self.circuit_base_instant.elapsed().as_millis() as u64
    }

    fn is_circuit_open(&self) -> bool {
        let last = self.circuit_last_error_ms.load(Ordering::Relaxed);
        !self.circuit_cooldown.is_zero()
            && last != 0
            && self.now_ms_since_base().saturating_sub(last)
                < self.circuit_cooldown.as_millis() as u64
    }

    fn open_circuit(&self) {
        self.circuit_last_error_ms
            .store(self.now_ms_since_base(), Ordering::Relaxed);
    }

    pub async fn project_data(&self, id: &str) -> RpcResult<ProjectDataWithLimits> {
        let time = Instant::now();
        let request = ProjectDataRequest::new(id).include_limits();
        let (source, data) = self.project_data_internal(request).await?;
        self.metrics.request(time.elapsed(), source, &data);
        let project_data = data?;
        Ok(ProjectDataWithLimits {
            data: project_data.data,
            limits: project_data.limits.unwrap_or(PlanLimits {
                tier: "".to_owned(),
                is_above_rpc_limit: false,
                is_above_mau_limit: false,
            }),
        })
    }

    pub async fn project_data_request(
        &self,
        request: ProjectDataRequest<'_>,
    ) -> RpcResult<ProjectDataResponse> {
        let time = Instant::now();
        let (source, data) = self.project_data_internal(request).await?;
        self.metrics.request(time.elapsed(), source, &data);
        Ok(data?)
    }

    async fn project_data_internal(
        &self,
        request: ProjectDataRequest<'_>,
    ) -> RpcResult<(ResponseSource, ProjectDataResult)> {
        if let Some(cache) = &self.cache {
            let time = Instant::now();
            let data = cache.fetch(request.id).await?;
            self.metrics.fetch_cache_time(time.elapsed());

            if let Some(data) = data {
                return Ok((ResponseSource::Cache, data));
            }
        }

        // Skip check if circuit breaker is open
        if self.is_circuit_open() {
            return Err(RpcError::ProjectDataError(
                ProjectDataError::RegistryTemporarilyUnavailable,
            ));
        }

        let id = request.id;
        let data = self.fetch_registry(request).await;

        // Cache all responses that we get
        let data = match data {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(ProjectDataError::NotFound),
            Err(RegistryError::Config(..)) => Err(ProjectDataError::RegistryConfigError),

            // This is an innternal error, we should not cache it and open the circuit breaker
            // to prevent the registry from being overwhelmed
            Err(err) => {
                error!("Error on fetching project registry API data: {:?}", err);
                self.open_circuit();
                return Err(RpcError::ProjectDataError(
                    ProjectDataError::RegistryTemporarilyUnavailable,
                ));
            }
        };

        if let Some(cache) = &self.cache {
            cache.set(id, &data).await;
        }

        Ok((ResponseSource::Registry, data))
    }

    async fn fetch_registry(
        &self,
        request: ProjectDataRequest<'_>,
    ) -> RegistryResult<Option<ProjectDataResponse>> {
        let time = Instant::now();

        let data = if let Some(client) = &self.client {
            client.project_data_with(request).await
        } else {
            Ok(Some(ProjectDataResponse {
                data: ProjectData {
                    uuid: "".to_owned(),
                    creator: "".to_owned(),
                    name: "".to_owned(),
                    push_url: None,
                    keys: vec![ProjectKey {
                        value: request.id.to_owned(),
                        is_valid: true,
                    }],
                    is_enabled: true,
                    is_verify_enabled: false,
                    is_rate_limited: false,
                    allowed_origins: vec![],
                    verified_domains: vec![],
                    bundle_ids: vec![],
                    package_names: vec![],
                },
                limits: Some(PlanLimits {
                    tier: "".to_owned(),
                    is_above_rpc_limit: false,
                    is_above_mau_limit: false,
                }),
                features: None,
            }))
        };
        self.metrics.fetch_registry_time(time.elapsed());
        data
    }
}

fn open_redis(
    addr: &redis::Addr<'_>,
    redis_max_connections: usize,
) -> Result<Arc<redis::Redis>, StorageError> {
    redis::Redis::new(addr, redis_max_connections).map(Arc::new)
}
