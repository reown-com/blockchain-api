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
    cerberus::project::ProjectData,
    ethers::types::H160,
    sqlx::PgPool,
    std::{collections::HashMap, sync::Arc},
    tap::TapFallible,
    tracing::info,
};

pub struct AppState {
    pub config: Config,
    pub postgres: PgPool,
    pub providers: ProviderRepository,
    pub metrics: Arc<Metrics>,
    pub registry: Registry,
    pub identity_cache: Option<Arc<dyn KeyValueStorage<IdentityResponse>>>,
    pub analytics: RPCAnalytics,
    pub compile_info: CompileInfo,
    pub ens_allowlist: Option<HashMap<H160, String>>,
    /// Service instance uptime measurement
    pub uptime: std::time::Instant,
}

#[allow(clippy::too_many_arguments)]
pub fn new_state(
    config: Config,
    postgres: PgPool,
    providers: ProviderRepository,
    metrics: Arc<Metrics>,
    registry: Registry,
    identity_cache: Option<Arc<dyn KeyValueStorage<IdentityResponse>>>,
    analytics: RPCAnalytics,
    ens_allowlist: Option<HashMap<H160, String>>,
) -> AppState {
    AppState {
        config,
        postgres,
        providers,
        metrics,
        registry,
        identity_cache,
        analytics,
        compile_info: CompileInfo {},
        ens_allowlist,
        uptime: std::time::Instant::now(),
    }
}

impl AppState {
    pub async fn update_provider_weights(&self) {
        self.providers.update_weights(&self.metrics).await;
    }

    #[tracing::instrument(skip(self))]
    async fn get_project_data_validated(&self, id: &str) -> Result<ProjectData, RpcError> {
        let project = self
            .registry
            .project_data(id)
            .await
            .tap_err(|_| self.metrics.add_rejected_project())?;

        project.validate_access(id, None).tap_err(|e| {
            self.metrics.add_rejected_project();
            info!("Denied access for project: {id}, with reason: {e}");
        })?;

        Ok(project)
    }

    pub async fn validate_project_access(&self, id: &str) -> Result<(), RpcError> {
        self.get_project_data_validated(id).await.map(drop)
    }

    #[tracing::instrument(skip(self))]
    pub async fn validate_project_access_and_quota(&self, id: &str) -> Result<(), RpcError> {
        let project = self.get_project_data_validated(id).await?;

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
