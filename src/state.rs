use {
    crate::{
        analytics::RPCAnalytics,
        env::Config,
        metrics::Metrics,
        project::Registry,
        providers::ProviderRepository,
        utils::build::CompileInfo,
    },
    opentelemetry_prometheus::PrometheusExporter,
};

pub struct AppState {
    pub config: Config,
    pub providers: ProviderRepository,
    pub exporter: PrometheusExporter,
    pub metrics: Metrics,
    pub registry: Registry,
    pub analytics: RPCAnalytics,
    pub compile_info: CompileInfo,
}

impl AppState {
    pub fn new(
        config: Config,
        providers: ProviderRepository,
        exporter: PrometheusExporter,
        metrics: Metrics,
        registry: Registry,
        analytics: RPCAnalytics,
    ) -> Self {
        Self {
            config,
            providers,
            exporter,
            metrics,
            registry,
            analytics,
            compile_info: CompileInfo {},
        }
    }

    #[cfg(feature = "dynamic-weights")]
    pub async fn update_provider_weights(&self) {
        self.providers.update_weights(&self.metrics).await;
    }
}
