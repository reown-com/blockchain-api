use {
    crate::project::{error::ProjectDataError, storage::ProjectDataResult, ResponseSource},
    std::time::Duration,
    wc::metrics::{counter, histogram, EnumLabel, StringLabel},
};

#[derive(Clone, Debug)]
pub struct ProjectDataMetrics {}

impl ProjectDataMetrics {
    pub fn new() -> Self {
        Self {}
    }

    pub fn fetch_cache_time(&self, time: Duration) {
        histogram!("project_data_local_cache_time").record(duration_ms(time));
    }

    pub fn fetch_registry_time(&self, time: Duration) {
        histogram!("project_data_registry_api_time").record(duration_ms(time));
    }

    pub fn request(&self, time: Duration, source: ResponseSource, resp: &ProjectDataResult) {
        counter!("project_data_requests_total",
            EnumLabel<"source", ResponseSource> => source,
            StringLabel<"response", String> => &response_tag(resp)
        )
        .increment(1);
        histogram!("project_data_total_time").record(duration_ms(time));
    }
}

#[inline]
fn response_tag(resp: &ProjectDataResult) -> String {
    match resp {
        Ok(_) => "ok".to_string(),
        Err(ProjectDataError::NotFound) => "not_found".to_string(),
        Err(ProjectDataError::RegistryConfigError) => "registry_config_error".to_string(),
        Err(ProjectDataError::RegistryTemporarilyUnavailable) => {
            "registry_temporarily_unavailable".to_string()
        }
    }
}

#[inline]
fn duration_ms(val: Duration) -> f64 {
    // Convert to milliseconds.
    val.as_secs_f64() * 1_000f64
}
