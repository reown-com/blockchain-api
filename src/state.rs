use {
    crate::{
        analytics::RPCAnalytics,
        env::Config,
        error::RpcError,
        handlers::identity::IdentityResponse,
        metrics::Metrics,
        project::Registry,
        providers::ProviderRepository,
        storage::KeyValueStorage,
        utils::build::CompileInfo,
    },
    std::sync::Arc,
    tap::TapFallible,
    tracing::info,
};

pub struct AppState {
    pub config: Config,
    pub providers: ProviderRepository,
    pub metrics: Arc<Metrics>,
    pub registry: Registry,
    pub identity_cache: Option<Arc<dyn KeyValueStorage<IdentityResponse>>>,
    pub analytics: RPCAnalytics,
    pub compile_info: CompileInfo,
}

pub fn new_state(
    config: Config,
    providers: ProviderRepository,
    metrics: Arc<Metrics>,
    registry: Registry,
    identity_cache: Option<Arc<dyn KeyValueStorage<IdentityResponse>>>,
    analytics: RPCAnalytics,
) -> AppState {
    AppState {
        config,
        providers,
        metrics,
        registry,
        identity_cache,
        analytics,
        compile_info: CompileInfo {},
    }
}

impl AppState {
    pub async fn update_provider_weights(&self) {
        self.providers.update_weights(&self.metrics).await;
    }

    pub async fn validate_project_access(&self, id: &str) -> Result<(), RpcError> {
        let project = self
            .registry
            .project_data(id)
            .await
            .tap_err(|_| self.metrics.add_rejected_project())?;

        project.validate_access(id, None).tap_err(|e| {
            self.metrics.add_rejected_project();
            info!("Denied access for project: {id}, with reason: {e}");
        })?;

        if !project.quota.is_valid {
            self.metrics.add_quota_limited_project();
            info!(
                project_id = id,
                max = project.quota.max,
                current = project.quota.current,
                "Quota limit reached"
            );
        }

        Ok(())
    }
}
