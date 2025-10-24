pub use config::*;
use {
    crate::{
        project::{error::ProjectDataError, metrics::ProjectDataMetrics},
        storage::{error::StorageError, KeyValueStorage, StorageResult},
    },
    cerberus::project::{ProjectDataRequest, ProjectDataResponse},
    std::{
        sync::Arc,
        time::{Duration, Instant},
    },
    tap::TapFallible,
    tracing::{error, warn},
};

mod config;

pub type ProjectDataResult = Result<ProjectDataResponse, ProjectDataError>;

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

    pub async fn fetch(
        &self,
        request: ProjectDataRequest<'_>,
    ) -> StorageResult<Option<ProjectDataResult>> {
        let time = Instant::now();

        let cache_key = build_cache_key(request);

        let data = match self.cache.get(&cache_key).await {
            Ok(data) => data,
            Err(StorageError::Deserialize(_)) => {
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

    pub async fn set(&self, request: ProjectDataRequest<'_>, data: &ProjectDataResult) {
        let cache_key = build_cache_key(request);

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
fn build_cache_key(request: ProjectDataRequest<'_>) -> String {
    let flags = (request.include_limits as u8) | ((request.include_features as u8) << 1);
    format!("project-data-v3/{}/{}", request.id, flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_cache_key_bitmask_values() {
        let id = "123e4567-e89b-12d3-a456-426614174000";

        let k0 = build_cache_key(ProjectDataRequest::new(id));
        assert_eq!(k0, format!("project-data-v3/{}/0", id));

        let k1 = build_cache_key(ProjectDataRequest::new(id).include_limits());
        assert_eq!(k1, format!("project-data-v3/{}/1", id));

        let k2 = build_cache_key(ProjectDataRequest::new(id).include_features());
        assert_eq!(k2, format!("project-data-v3/{}/2", id));

        let k3 = build_cache_key(
            ProjectDataRequest::new(id)
                .include_limits()
                .include_features(),
        );
        assert_eq!(k3, format!("project-data-v3/{}/3", id));
    }

    #[test]
    fn build_cache_key_uniqueness() {
        let id = "abc";
        let mut set = std::collections::HashSet::new();
        set.insert(build_cache_key(ProjectDataRequest::new(id)));
        set.insert(build_cache_key(
            ProjectDataRequest::new(id).include_limits(),
        ));
        set.insert(build_cache_key(
            ProjectDataRequest::new(id).include_features(),
        ));
        set.insert(build_cache_key(
            ProjectDataRequest::new(id)
                .include_limits()
                .include_features(),
        ));
        assert_eq!(set.len(), 4);
    }
}
