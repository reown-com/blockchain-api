use {
    crate::project::{error::ProjectDataError, storage::ProjectDataResult, ResponseSource},
    std::time::Duration,
    wc::metrics::otel::{
        self,
        metrics::{Counter, Histogram, Meter},
        KeyValue,
    },
};

const METRIC_NAMESPACE: &str = "project_data";

#[derive(Clone, Debug)]
pub struct ProjectDataMetrics {
    requests_total: Counter<u64>,
    registry_api_time: Histogram<f64>,
    local_cache_time: Histogram<f64>,
    total_time: Histogram<f64>,
}

impl ProjectDataMetrics {
    pub fn new(meter: &Meter) -> Self {
        let requests_total = meter
            .u64_counter(create_counter_name("requests_total"))
            .with_description("Total number of project data requests")
            .init();

        let registry_api_time = meter
            .f64_histogram(create_counter_name("registry_api_time"))
            .with_description("Average latency of the registry API fetching")
            .init();

        let local_cache_time = meter
            .f64_histogram(create_counter_name("local_cache_time"))
            .with_description("Average latency of the local cache fetching")
            .init();

        let total_time = meter
            .f64_histogram(create_counter_name("total_time"))
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
        self.local_cache_time
            .record(&otel::Context::new(), duration_ms(time), &[]);
    }

    pub fn fetch_registry_time(&self, time: Duration) {
        self.registry_api_time
            .record(&otel::Context::new(), duration_ms(time), &[]);
    }

    pub fn request(&self, time: Duration, source: ResponseSource, resp: &ProjectDataResult) {
        self.requests_total.add(&otel::Context::new(), 1, &[
            source_tag(source),
            response_tag(resp),
        ]);
        self.total_time
            .record(&otel::Context::new(), duration_ms(time), &[]);
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
