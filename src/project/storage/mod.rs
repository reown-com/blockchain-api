pub use config::*;
use {
    crate::{
        project::{error::ProjectDataError, metrics::ProjectDataMetrics},
        storage::{error::StorageError, KeyValueStorage, StorageResult},
    },
    cerberus::project::ProjectData,
    std::{
        sync::Arc,
        time::{Duration, Instant},
    },
    tap::TapFallible,
    tracing::{error, warn},
};

mod config;

pub type ProjectDataResult = Result<ProjectData, ProjectDataError>;

#[derive(Clone, Debug)]
pub struct ProjectStorage {
    cache: Arc<dyn KeyValueStorage<ProjectDataResult>>,
    cache_ttl: Duration,
    metrics: ProjectDataMetrics,
}

impl ProjectStorage {
    pub fn new(
        cache: Arc<dyn KeyValueStorage<ProjectDataResult>>,
        cache_ttl: Duration,
        metrics: ProjectDataMetrics,
    ) -> Self {
        ProjectStorage {
            cache,
            cache_ttl,
            metrics,
        }
    }

    pub async fn fetch(&self, id: &str) -> StorageResult<Option<ProjectDataResult>> {
        let time = Instant::now();

        let cache_key = build_cache_key(id);

        let data = match self.cache.get(&cache_key).await {
            Ok(data) => data,
            Err(StorageError::Deserialize) => {
                warn!("failed to deserialize cached ProjectData");
                None
            }
            Err(err) => {
                warn!(?err, "error fetching data from project data cache");
                return Err(err);
            }
        };

        self.metrics.fetch_cache_time(time.elapsed());

        Ok(data)
    }

    pub async fn set(&self, id: &str, data: &ProjectDataResult) {
        let cache_key = build_cache_key(id);

        let serialized = match crate::storage::serialize(&data) {
            Ok(serialized) => serialized,
            Err(err) => {
                error!(?err, "failed to serialize cached project data");
                return;
            }
        };
        let cache = self.cache.clone();
        let cache_ttl = self.cache_ttl;

        // Do not block on cache write.
        tokio::spawn(async move {
            cache
                .set_serialized(&cache_key, &serialized, Some(cache_ttl))
                .await
                .tap_err(|err| warn!("failed to cache project data: {:?}", err))
                .ok();
        });
    }
}

#[inline]
fn build_cache_key(id: &str) -> String {
    format!("project-data/{id}")
}
