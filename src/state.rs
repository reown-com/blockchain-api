use crate::analytics::RPCAnalytics;
use crate::metrics::Metrics;
use crate::project::Registry;
use crate::providers::ProviderRepository;
use crate::BuildInfo;
use crate::Config;
use opentelemetry_prometheus::PrometheusExporter;

pub struct State {
    pub config: Config,
    pub providers: ProviderRepository,
    pub exporter: PrometheusExporter,
    pub metrics: Metrics,
    pub registry: Registry,
    pub analytics: RPCAnalytics,
    pub build_info: BuildInfo,
}

build_info::build_info!(fn build_info);

pub fn new_state(
    config: Config,
    providers: ProviderRepository,
    exporter: PrometheusExporter,
    metrics: Metrics,
    registry: Registry,
    analytics: RPCAnalytics,
) -> State {
    let build_info: &BuildInfo = build_info();

    State {
        config,
        providers,
        exporter,
        metrics,
        registry,
        analytics,
        build_info: build_info.clone(),
    }
}
