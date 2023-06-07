use {
    crate::providers::{ProviderKind, RpcProvider},
    hyper::http,
    opentelemetry::metrics::{Counter, Meter, ValueRecorder},
};

#[derive(Debug)]
pub struct Metrics {
    pub rpc_call_counter: Counter<u64>,
    pub http_call_counter: Counter<u64>,
    pub provider_finished_call_counter: Counter<u64>,
    pub provider_failed_call_counter: Counter<u64>,
    pub http_latency_tracker: ValueRecorder<f64>,
    pub http_external_latency_tracker: ValueRecorder<f64>,
    pub rejected_project_counter: Counter<u64>,
    pub rate_limited_call_counter: Counter<u64>,
    pub provider_status_code_counter: Counter<u64>,
    pub weights_value_recorder: ValueRecorder<u64>,
}

impl Metrics {
    pub fn new(meter: &Meter) -> Self {
        let rpc_call_counter = meter
            .u64_counter("rpc_call_counter")
            .with_description("The number of rpc calls served")
            .init();

        let http_call_counter = meter
            .u64_counter("http_call_counter")
            .with_description("The number of http calls served")
            .init();

        let http_latency_tracker = meter
            .f64_value_recorder("http_latency_tracker")
            .with_description("The http call latency")
            .init();

        let http_external_latency_tracker = meter
            .f64_value_recorder("http_external_latency_tracker")
            .with_description("The http call latency for external providers")
            .init();

        let rejected_project_counter = meter
            .u64_counter("rejected_project_counter")
            .with_description("The number of calls for invalid project ids")
            .init();

        let rate_limited_call_counter = meter
            .u64_counter("rate_limited_counter")
            .with_description("The number of calls that got rate limited")
            .init();

        let provider_finished_call_counter = meter
            .u64_counter("provider_finished_call_counter")
            .with_description("The number of calls to provider that finished successfully")
            .init();

        let provider_failed_call_counter = meter
            .u64_counter("provider_failed_call_counter")
            .with_description("The number of calls to provider that failed")
            .init();

        let provider_status_code_counter = meter
            .u64_counter("provider_status_code_counter")
            .with_description("The count of status codes returned by providers")
            .init();

        let weights_value_recorder = meter
            .u64_value_recorder("provider_weights")
            .with_description("The weights of the providers")
            .init();

        Metrics {
            rpc_call_counter,
            http_call_counter,
            http_external_latency_tracker,
            http_latency_tracker,
            rejected_project_counter,
            rate_limited_call_counter,
            provider_failed_call_counter,
            provider_finished_call_counter,
            provider_status_code_counter,
            weights_value_recorder,
        }
    }
}

impl Metrics {
    pub fn add_rpc_call(&self, chain_id: &str) {
        self.rpc_call_counter.add(1, &[opentelemetry::KeyValue::new(
            "chain.id",
            chain_id.to_owned(),
        )]);
    }

    pub fn add_http_call(&self, code: u16, route: &str) {
        self.http_call_counter.add(1, &[
            opentelemetry::KeyValue::new("code", i64::from(code)),
            opentelemetry::KeyValue::new("route", route.to_owned()),
        ]);
    }

    pub fn add_http_latency(&self, code: u16, route: &str, latency: f64) {
        self.http_latency_tracker.record(latency, &[
            opentelemetry::KeyValue::new("code", i64::from(code)),
            opentelemetry::KeyValue::new("route", route.to_owned()),
        ])
    }

    pub fn add_external_http_latency(&self, provider_kind: ProviderKind, latency: f64) {
        self.http_external_latency_tracker
            .record(latency, &[opentelemetry::KeyValue::new(
                "provider",
                provider_kind.to_string(),
            )])
    }

    pub fn add_rejected_project(&self) {
        self.rejected_project_counter.add(1, &[])
    }

    pub fn add_rate_limited_call(&self, provider: &dyn RpcProvider, project_id: String) {
        self.rate_limited_call_counter.add(1, &[
            opentelemetry::KeyValue::new("provider_kind", provider.provider_kind().to_string()),
            opentelemetry::KeyValue::new("project_id", project_id),
        ])
    }

    pub fn add_failed_provider_call(&self, provider: &dyn RpcProvider) {
        self.provider_failed_call_counter
            .add(1, &[opentelemetry::KeyValue::new(
                "provider",
                provider.provider_kind().to_string(),
            )])
    }

    pub fn add_finished_provider_call(&self, provider: &dyn RpcProvider) {
        self.provider_finished_call_counter
            .add(1, &[opentelemetry::KeyValue::new(
                "provider",
                provider.provider_kind().to_string(),
            )])
    }

    pub fn add_status_code_for_provider(
        &self,
        provider: &dyn RpcProvider,
        status: http::StatusCode,
        chain_id: &str,
        project_id: &str,
    ) {
        self.provider_status_code_counter.add(1, &[
            opentelemetry::KeyValue::new("provider", provider.provider_kind().to_string()),
            opentelemetry::KeyValue::new("status_code", format!("{}", status.as_u16())),
            opentelemetry::KeyValue::new("chain_id", chain_id.to_owned()),
            opentelemetry::KeyValue::new("project_id", project_id.to_owned()),
        ])
    }

    #[cfg(feature = "dynamic-weights")]
    pub fn record_provider_weight(&self, provider: &ProviderKind, chain_id: &str, weight: u64) {
        self.weights_value_recorder.record(weight, &[
            opentelemetry::KeyValue::new("provider", provider.to_string()),
            opentelemetry::KeyValue::new("chain_id", chain_id.to_string()),
        ])
    }
}
