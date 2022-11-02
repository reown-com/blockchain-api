use {common::metrics::AppMetrics, opentelemetry::metrics::ValueRecorder, std::time::Duration};
use {opentelemetry::metrics::Counter, opentelemetry::KeyValue};

use crate::project::error::ProjectDataError;
use crate::project::storage::ProjectDataResult;
use crate::project::ResponseSource;

const METRIC_NAMESPACE: &str = "project_data";

#[derive(Clone, Debug)]
pub struct ProjectDataMetrics {
    requests_total: Counter<u64>,
    registry_api_time: ValueRecorder<f64>,
    local_cache_time: ValueRecorder<f64>,
    total_time: ValueRecorder<f64>,
}

impl ProjectDataMetrics {
    pub fn new(app_metrics: &AppMetrics) -> Self {
        let requests_total = app_metrics
            .meter()
            .u64_counter(create_counter_name("requests_total"))
            .with_description("Total number of project data requests")
            .init();

        let registry_api_time = app_metrics
            .meter()
            .f64_value_recorder(create_counter_name("registry_api_time"))
            .with_description("Average latency of the registry API fetching")
            .init();

        let local_cache_time = app_metrics
            .meter()
            .f64_value_recorder(create_counter_name("local_cache_time"))
            .with_description("Average latency of the local cache fetching")
            .init();

        let total_time = app_metrics
            .meter()
            .f64_value_recorder(create_counter_name("total_time"))
            .with_description("Average total latency for project data fetching")
            .init();

        Self {
            requests_total,
            registry_api_time,
            local_cache_time,
            total_time,
        }
    }

    pub fn fetch_cache_time(&self, time: Duration) {
        self.local_cache_time.record(duration_ms(time), &[]);
    }

    pub fn fetch_registry_time(&self, time: Duration) {
        self.registry_api_time.record(duration_ms(time), &[]);
    }

    pub fn request(&self, time: Duration, source: ResponseSource, resp: &ProjectDataResult) {
        self.requests_total
            .add(1, &[source_tag(source), response_tag(resp)]);
        self.total_time.record(duration_ms(time), &[]);
    }
}

fn source_tag(source: ResponseSource) -> KeyValue {
    let value = match source {
        ResponseSource::Cache => "cache",
        ResponseSource::Registry => "registry",
    };

    KeyValue::new("source", value)
}

fn response_tag(resp: &ProjectDataResult) -> KeyValue {
    let value = match resp {
        Ok(_) => "ok",
        Err(ProjectDataError::NotFound) => "not_found",
        Err(ProjectDataError::RegistryConfigError) => "registry_config_error",
    };

    KeyValue::new("response", value)
}

#[inline]
fn create_counter_name(name: &str) -> String {
    format!("{METRIC_NAMESPACE}_{name}")
}

#[inline]
fn duration_ms(val: Duration) -> f64 {
    // Convert to milliseconds.
    val.as_secs_f64() * 1_000f64
}
