use crate::analytics::RPCAnalytics;
use crate::metrics::Metrics;
use crate::project::Registry;
use crate::providers::ProviderRepository;
use crate::utils::build::CompileInfo;
use crate::Config;
use opentelemetry_prometheus::PrometheusExporter;

pub struct State {
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
) -> State {
    State {
        config,
        providers,
        exporter,
        metrics,
        registry,
        analytics,
        compile_info: CompileInfo {},
    }
}
