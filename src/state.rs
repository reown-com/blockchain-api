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
    pub prometheus_client: prometheus_http_query::Client,
}

pub fn new_state(
    config: Config,
    providers: ProviderRepository,
    exporter: PrometheusExporter,
    metrics: Metrics,
    registry: Registry,
    analytics: RPCAnalytics,
) -> AppState {
    let client = prometheus_http_query::Client::try_from(config.prometheus_query_url.clone())
        .expect("Failed to connect to prometheus");

    AppState {
        prometheus_client: client,
        config,
        providers,
        exporter,
        metrics,
        registry,
        analytics,
        compile_info: CompileInfo {},
    }
}

impl AppState {
    pub async fn update_provider_weights(&self) {
        self.providers.update_weights().await;
    }
}
