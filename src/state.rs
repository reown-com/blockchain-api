use {
    crate::{
        analytics::RPCAnalytics,
        env::Config,
        error::RpcError,
        handlers::{balance::BalanceResponseBody, identity::IdentityResponse},
        metrics::Metrics,
        project::Registry,
        providers::ProviderRepository,
        storage::{irn::Irn, KeyValueStorage},
        utils::{build::CompileInfo, rate_limit::RateLimit},
    },
    cerberus::project::ProjectDataWithLimits,
    moka::future::Cache,
    sqlx::PgPool,
    std::sync::Arc,
    tap::TapFallible,
    tracing::debug,
};

pub struct AppState {
    pub config: Config,
    pub postgres: PgPool,
    pub providers: ProviderRepository,
    pub metrics: Arc<Metrics>,
    pub registry: Registry,
    pub analytics: RPCAnalytics,
    pub compile_info: CompileInfo,
    /// Service instance uptime measurement
    pub uptime: std::time::Instant,
    /// Shared http client
    pub http_client: reqwest::Client,
    // Rate limiting checks
    pub rate_limit: Option<RateLimit>,
    // IRN client
    pub irn: Option<Irn>,
    // Redis caching
    pub identity_cache: Option<Arc<dyn KeyValueStorage<IdentityResponse>>>,
    pub balance_cache: Option<Arc<dyn KeyValueStorage<BalanceResponseBody>>>,
    // Moka local instance in-memory cache
    pub moka_cache: Cache<String, String>,
}

#[allow(clippy::too_many_arguments)]
pub fn new_state(
    config: Config,
    postgres: PgPool,
    providers: ProviderRepository,
    metrics: Arc<Metrics>,
    registry: Registry,
    analytics: RPCAnalytics,
    http_client: reqwest::Client,
    rate_limit: Option<RateLimit>,
    irn: Option<Irn>,
    identity_cache: Option<Arc<dyn KeyValueStorage<IdentityResponse>>>,
    balance_cache: Option<Arc<dyn KeyValueStorage<BalanceResponseBody>>>,
) -> AppState {
    let moka_cache = Cache::builder().build();
    AppState {
        config,
        postgres,
        providers,
        metrics,
        registry,
        analytics,
        compile_info: CompileInfo {},
        uptime: std::time::Instant::now(),
        http_client,
        rate_limit,
        irn,
        identity_cache,
        balance_cache,
        moka_cache,
    }
}

impl AppState {
    pub async fn update_provider_weights(&self) {
        self.providers.update_weights(&self.metrics).await;
    }

    #[tracing::instrument(skip(self), level = "debug")]
    async fn get_project_data_validated(
        &self,
        id: &str,
    ) -> Result<ProjectDataWithLimits, RpcError> {
        let project = self.registry.project_data(id).await.tap_err(|e| {
            debug!("Denied access for project: {id}, with reason: {e}");
            self.metrics.add_rejected_project();
        })?;

        project.data.validate_access(id, None).tap_err(|e| {
            debug!("Denied access for project: {id}, with reason: {e}");
            self.metrics.add_rejected_project();
        })?;

        Ok(project)
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn validate_project_access(&self, id: &str) -> Result<(), RpcError> {
        if !self.config.server.validate_project_id {
            return Ok(());
        }

        self.get_project_data_validated(id).await.map(drop)
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub async fn validate_project_access_and_quota(&self, id: &str) -> Result<(), RpcError> {
        if !self.config.server.validate_project_id {
            return Ok(());
        }

        let project = self.get_project_data_validated(id).await?;

        validate_project_quota(&project).tap_err(|e| {
            debug!(
                project_id = id,
                is_above_rpc_limit = project.limits.is_above_rpc_limit,
                error = ?e,
                "Quota limit reached"
            );
            self.metrics.add_quota_limited_project();
        })
    }
}

#[tracing::instrument(level = "debug")]
fn validate_project_quota(project_data: &ProjectDataWithLimits) -> Result<(), RpcError> {
    if !project_data.limits.is_above_rpc_limit {
        Ok(())
    } else {
        Err(RpcError::QuotaLimitReached)
    }
}

#[cfg(test)]
mod test {
    use {
        super::{ProjectDataWithLimits, RpcError},
        cerberus::project::{PlanLimits, ProjectData},
    };

    #[test]
    fn validate_project_quota() {
        // TODO: Handle this in some stub implementation of "Registry" abstraction.
        let mut project = ProjectDataWithLimits {
            data: ProjectData {
                uuid: "".to_owned(),
                creator: "".to_owned(),
                name: "".to_owned(),
                push_url: None,
                keys: vec![],
                is_enabled: true,
                is_verify_enabled: false,
                is_rate_limited: false,
                allowed_origins: vec![],
                verified_domains: vec![],
                bundle_ids: vec![],
                package_names: vec![],
            },
            limits: PlanLimits {
                tier: "".to_owned(),
                is_above_mau_limit: false,
                is_above_rpc_limit: false,
            },
        };

        match super::validate_project_quota(&project) {
            Ok(()) => {}
            res => panic!("Invalid result: {res:?}"),
        }

        project.limits.is_above_rpc_limit = true;
        match super::validate_project_quota(&project) {
            Err(RpcError::QuotaLimitReached) => {}
            res => panic!("Invalid result: {res:?}"),
        }
    }
}
