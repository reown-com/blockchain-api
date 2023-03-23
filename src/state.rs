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

pub fn new_state(
    config: Config,
    providers: ProviderRepository,
    exporter: PrometheusExporter,
    metrics: Metrics,
    registry: Registry,
    analytics: RPCAnalytics,
) -> AppState {
    AppState {
        config,
        providers,
        exporter,
        metrics,
        registry,
        analytics,
        compile_info: CompileInfo {},
    }
}
