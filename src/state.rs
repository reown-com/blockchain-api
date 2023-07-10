use {
    crate::{
        analytics::RPCAnalytics,
        env::Config,
        handlers::identity::IdentityResponse,
        metrics::Metrics,
        project::Registry,
        providers::ProviderRepository,
        storage::KeyValueStorage,
        utils::build::CompileInfo,
    },
    std::sync::Arc,
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
}
