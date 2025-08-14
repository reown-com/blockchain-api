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
    std::{sync::Arc, time::Instant},
    wc::metrics::ServiceMetrics,
};
pub use {config::*, error::*};

mod config;
mod error;

pub mod metrics;
pub mod storage;

#[derive(Debug, Clone)]
pub struct Registry {
    client: Option<RegistryHttpClient>,
    cache: Option<ProjectStorage>,
    metrics: ProjectDataMetrics,
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
        })
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

        let id = request.id;
        let data = self.fetch_registry(request).await;

        // Cache all responses that we get, even errors.
        let data = match data {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(ProjectDataError::NotFound),
            Err(RegistryError::Config(..)) => Err(ProjectDataError::RegistryConfigError),

            // This is a retryable error, don't cache the result.
            Err(err) => return Err(err.into()),
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
