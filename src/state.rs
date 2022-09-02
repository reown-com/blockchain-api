use crate::{BuildInfo, Config};
use opentelemetry::metrics::Counter;
use opentelemetry_prometheus::PrometheusExporter;

pub struct State {
    pub config: Config,
    pub exporter: Option<PrometheusExporter>,
    pub metrics: Option<Metrics>,
    pub build_info: BuildInfo,
}

pub struct Metrics {
    pub rpc_call_counter: Counter<u64>,
}

build_info::build_info!(fn build_info);

pub fn new_state(config: Config) -> State {
    let build_info: &BuildInfo = build_info();

    State {
        config,
        exporter: None,
        metrics: None,
        build_info: build_info.clone(),
    }
}
impl State {
    pub fn set_metrics(&mut self, exporter: PrometheusExporter, metrics: Metrics) {
        self.exporter = Some(exporter);
        self.metrics = Some(metrics);
    }
}
