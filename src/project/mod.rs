use cerberus::project::ProjectData;
use std::sync::Arc;
use std::time::Instant;

use cerberus::registry::{RegistryClient, RegistryError, RegistryHttpClient, RegistryResult};
use common::metrics::AppMetrics;
use common::storage::{redis, StorageError, StorageResult};

pub use config::*;
pub use error::*;

use crate::error::{RpcError, RpcResult};
use crate::project::metrics::ProjectDataMetrics;
use crate::project::storage::ProjectStorage;
use crate::project::storage::{Config as StorageConfig, ProjectDataResult};

mod config;
mod error;

pub mod metrics;
pub mod storage;

#[derive(Debug, Clone)]
pub struct Registry {
    client: RegistryHttpClient,
    cache: ProjectStorage,
    metrics: ProjectDataMetrics,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ResponseSource {
    Cache,
    Registry,
}

impl Registry {
    pub fn new(cfg_registry: &Config, cfg_storage: &StorageConfig) -> RpcResult<Self> {
        let api_url = &cfg_registry.api_url;
        let api_auth_token = &cfg_registry.api_auth_token;

        let client = if let (Some(api_url), Some(api_auth_token)) = (api_url, api_auth_token) {
            RegistryHttpClient::new(api_url, api_auth_token)?
        } else {
            return Err(RpcError::InvalidConfiguration(
                "missing registry api parameters".to_string(),
            ));
        };

        let metrics = ProjectDataMetrics::new(&AppMetrics::new(crate::PROXY_METRICS_NAME));

        let cache = open_redis(
            &cfg_storage.project_data_redis_addr(),
            cfg_storage.redis_max_connections,
        )?;
        let cache = ProjectStorage::new(
            cache,
            cfg_registry.project_data_cache_ttl(),
            metrics.clone(),
        );

        Ok(Self {
            client,
            cache,
            metrics,
        })
    }

    pub async fn project_data(&self, id: &str) -> RpcResult<ProjectData> {
        let time = Instant::now();
        let (source, data) = self.project_data_internal(id).await?;
        self.metrics.request(time.elapsed(), source, &data);
        Ok(data?)
    }

    async fn project_data_internal(
        &self,
        id: &str,
    ) -> RpcResult<(ResponseSource, ProjectDataResult)> {
        let data = self.fetch_cache(id).await?;

        if let Some(data) = data {
            return Ok((ResponseSource::Cache, data));
        }

        let data = self.fetch_registry(id).await;

        // Cache all responses that we get, even errors.
        let data = match data {
            Ok(Some(data)) => Ok(data),
            Ok(None) => Err(ProjectDataError::NotFound),
            Err(RegistryError::Config(..)) => Err(ProjectDataError::RegistryConfigError),

            // This is a retryable error, don't cache the result.
            Err(err) => return Err(err.into()),
        };

        self.cache.set(id, &data).await;

        Ok((ResponseSource::Registry, data))
    }

    async fn fetch_cache(&self, id: &str) -> StorageResult<Option<ProjectDataResult>> {
        let time = Instant::now();
        let data = self.cache.fetch(id).await;
        self.metrics.fetch_cache_time(time.elapsed());

        data
    }

    async fn fetch_registry(&self, id: &str) -> RegistryResult<Option<ProjectData>> {
        let time = Instant::now();
        let data = self.client.project_data(id).await;
        self.metrics.fetch_registry_time(time.elapsed());

        data
    }
}

fn open_redis(
    addr: &redis::Addr<'_>,
    redis_max_connections: usize,
) -> anyhow::Result<Arc<redis::Redis>, StorageError> {
    redis::Redis::new(addr, redis_max_connections).map(Arc::new)
}
