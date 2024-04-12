use {
    crate::{
        handlers::identity::IdentityLookupSource,
        providers::{ProviderKind, RpcProvider},
    },
    hyper::http,
    std::time::{Duration, SystemTime},
    sysinfo::{
        CpuRefreshKind,
        MemoryRefreshKind,
        RefreshKind,
        System,
        MINIMUM_CPU_UPDATE_INTERVAL,
    },
    wc::metrics::{
        otel::{
            self,
            metrics::{Counter, Histogram},
        },
        ServiceMetrics,
    },
};

#[derive(Debug)]
pub struct Metrics {
    pub rpc_call_counter: Counter<u64>,
    pub rpc_call_retries: Histogram<u64>,
    pub http_call_counter: Counter<u64>,
    pub provider_finished_call_counter: Counter<u64>,
    pub provider_failed_call_counter: Counter<u64>,
    pub http_latency_tracker: Histogram<f64>,
    pub http_external_latency_tracker: Histogram<f64>,
    pub rejected_project_counter: Counter<u64>,
    pub quota_limited_project_counter: Counter<u64>,
    pub rate_limited_call_counter: Counter<u64>,
    pub provider_status_code_counter: Counter<u64>,
    pub weights_value_recorder: Histogram<u64>,
    pub identity_lookup_latency_tracker: Histogram<f64>,
    pub identity_lookup_counter: Counter<u64>,
    pub identity_lookup_success_counter: Counter<u64>,
    pub identity_lookup_cache_latency_tracker: Histogram<f64>,
    pub identity_lookup_name_counter: Counter<u64>,
    pub identity_lookup_name_success_counter: Counter<u64>,
    pub identity_lookup_name_latency_tracker: Histogram<f64>,
    pub identity_lookup_avatar_counter: Counter<u64>,
    pub identity_lookup_avatar_success_counter: Counter<u64>,
    pub identity_lookup_avatar_latency_tracker: Histogram<f64>,
    pub identity_lookup_avatar_present_counter: Counter<u64>,
    pub identity_lookup_name_present_counter: Counter<u64>,
    pub websocket_connection_counter: Counter<u64>,
    pub history_lookup_counter: Counter<u64>,
    pub history_lookup_success_counter: Counter<u64>,
    pub history_lookup_latency_tracker: Histogram<f64>,

    // System metrics
    pub cpu_usage: Histogram<f64>,
    pub memory_total: Histogram<f64>,
    pub memory_used: Histogram<f64>,

    // Rate limiting
    pub rate_limited_entries_counter: Histogram<u64>,
}

impl Metrics {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let meter = ServiceMetrics::meter();

        let rpc_call_counter = meter
            .u64_counter("rpc_call_counter")
            .with_description("The number of rpc calls served")
            .init();

        let rpc_call_retries = meter
            .u64_histogram("rpc_call_retries")
            .with_description("Retries per RPC call")
            .init();

        let http_call_counter = meter
            .u64_counter("http_call_counter")
            .with_description("The number of http calls served")
            .init();

        let http_latency_tracker = meter
            .f64_histogram("http_latency_tracker")
            .with_description("The http call latency")
            .init();

        let http_external_latency_tracker = meter
            .f64_histogram("http_external_latency_tracker")
            .with_description("The http call latency for external providers")
            .init();

        let rejected_project_counter = meter
            .u64_counter("rejected_project_counter")
            .with_description("The number of calls for invalid project ids")
            .init();

        let quota_limited_project_counter = meter
            .u64_counter("quota_limited_project_counter")
            .with_description("The number of calls for quota limited project ids")
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
            .u64_histogram("provider_weights")
            .with_description("The weights of the providers")
            .init();

        let identity_lookup_counter = meter
            .u64_counter("identity_lookup_counter")
            .with_description("The number of identity lookups served")
            .init();

        let identity_lookup_success_counter = meter
            .u64_counter("identity_lookup_success_counter")
            .with_description("The number of identity lookups that were successful")
            .init();

        let identity_lookup_latency_tracker = meter
            .f64_histogram("identity_lookup_latency_tracker")
            .with_description("The latency to serve identity lookups")
            .init();

        let identity_lookup_cache_latency_tracker = meter
            .f64_histogram("identity_lookup_cache_latency_tracker")
            .with_description("The latency to lookup identity in the cache")
            .init();

        let identity_lookup_name_counter = meter
            .u64_counter("identity_lookup_name_counter")
            .with_description("The number of name lookups")
            .init();

        let identity_lookup_name_success_counter = meter
            .u64_counter("identity_lookup_name_success_counter")
            .with_description("The number of name lookups that were successfull")
            .init();

        let identity_lookup_name_latency_tracker = meter
            .f64_histogram("identity_lookup_name_latency_tracker")
            .with_description("The latency of performing the name lookup")
            .init();

        let identity_lookup_avatar_counter = meter
            .u64_counter("identity_lookup_avatar_counter")
            .with_description("The number of avatar lookups")
            .init();

        let identity_lookup_avatar_success_counter = meter
            .u64_counter("identity_lookup_avatar_success_counter")
            .with_description("The number of avatar lookups that were successfull")
            .init();

        let identity_lookup_avatar_latency_tracker = meter
            .f64_histogram("identity_lookup_avatar_latency_tracker")
            .with_description("The latency of performing the avatar lookup")
            .init();

        let identity_lookup_name_present_counter = meter
            .u64_counter("identity_lookup_name_present_counter")
            .with_description("The number of identity lookups that returned a name")
            .init();

        let identity_lookup_avatar_present_counter = meter
            .u64_counter("identity_lookup_avatar_present_counter")
            .with_description("The number of identity lookups that returned an avatar")
            .init();

        let websocket_connection_counter = meter
            .u64_counter("websocket_connection_counter")
            .with_description("The number of websocket connections")
            .init();

        let history_lookup_counter = meter
            .u64_counter("history_lookup_counter")
            .with_description("The number of transaction history lookups")
            .init();

        let history_lookup_success_counter = meter
            .u64_counter("history_lookup_success_counter")
            .with_description("The number of transaction history that were successfull")
            .init();

        let history_lookup_latency_tracker = meter
            .f64_histogram("history_lookup_latency_tracker")
            .with_description("The latency to serve transactions history lookups")
            .init();

        let cpu_usage = meter
            .f64_histogram("cpu_usage")
            .with_description("The cpu(s) usage")
            .init();

        let memory_total = meter
            .f64_histogram("memory_total")
            .with_description("Total system memory")
            .init();

        let memory_used = meter
            .f64_histogram("memory_used")
            .with_description("Used system memory")
            .init();

        let rate_limited_entries_counter = meter
            .u64_histogram("rate_limited_entries")
            .with_description("The rate limited entries counter")
            .init();

        Metrics {
            rpc_call_counter,
            rpc_call_retries,
            http_call_counter,
            http_external_latency_tracker,
            http_latency_tracker,
            rejected_project_counter,
            quota_limited_project_counter,
            rate_limited_call_counter,
            provider_failed_call_counter,
            provider_finished_call_counter,
            provider_status_code_counter,
            weights_value_recorder,
            identity_lookup_counter,
            identity_lookup_success_counter,
            identity_lookup_latency_tracker,
            identity_lookup_cache_latency_tracker,
            identity_lookup_name_counter,
            identity_lookup_name_success_counter,
            identity_lookup_name_latency_tracker,
            identity_lookup_avatar_counter,
            identity_lookup_avatar_success_counter,
            identity_lookup_avatar_latency_tracker,
            identity_lookup_name_present_counter,
            identity_lookup_avatar_present_counter,
            websocket_connection_counter,
            history_lookup_counter,
            history_lookup_success_counter,
            history_lookup_latency_tracker,
            cpu_usage,
            memory_total,
            memory_used,
            rate_limited_entries_counter,
        }
    }
}

impl Metrics {
    pub fn add_rpc_call(&self, chain_id: String) {
        self.rpc_call_counter
            .add(&otel::Context::new(), 1, &[otel::KeyValue::new(
                "chain.id", chain_id,
            )]);
    }

    pub fn add_rpc_call_retries(&self, retires_count: u64, chain_id: String) {
        self.rpc_call_retries
            .record(&otel::Context::new(), retires_count, &[
                otel::KeyValue::new("chain_id", chain_id),
            ])
    }

    pub fn add_http_call(&self, code: u16, route: String) {
        self.http_call_counter.add(&otel::Context::new(), 1, &[
            otel::KeyValue::new("code", i64::from(code)),
            otel::KeyValue::new("route", route),
        ]);
    }

    pub fn add_http_latency(&self, code: u16, route: String, latency: f64) {
        self.http_latency_tracker
            .record(&otel::Context::new(), latency, &[
                otel::KeyValue::new("code", i64::from(code)),
                otel::KeyValue::new("route", route),
            ])
    }

    pub fn add_external_http_latency(&self, provider_kind: ProviderKind, latency: f64) {
        self.http_external_latency_tracker
            .record(&otel::Context::new(), latency, &[otel::KeyValue::new(
                "provider",
                provider_kind.to_string(),
            )])
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn add_rejected_project(&self) {
        self.rejected_project_counter
            .add(&otel::Context::new(), 1, &[])
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn add_quota_limited_project(&self) {
        self.quota_limited_project_counter
            .add(&otel::Context::new(), 1, &[])
    }

    pub fn add_rate_limited_call(&self, provider: &dyn RpcProvider, project_id: String) {
        self.rate_limited_call_counter
            .add(&otel::Context::new(), 1, &[
                otel::KeyValue::new("provider_kind", provider.provider_kind().to_string()),
                otel::KeyValue::new("project_id", project_id),
            ])
    }

    pub fn add_failed_provider_call(&self, provider: &dyn RpcProvider) {
        self.provider_failed_call_counter
            .add(&otel::Context::new(), 1, &[otel::KeyValue::new(
                "provider",
                provider.provider_kind().to_string(),
            )])
    }

    pub fn add_finished_provider_call(&self, provider: &dyn RpcProvider) {
        self.provider_finished_call_counter
            .add(&otel::Context::new(), 1, &[otel::KeyValue::new(
                "provider",
                provider.provider_kind().to_string(),
            )])
    }

    pub fn add_status_code_for_provider(
        &self,
        provider: &dyn RpcProvider,
        status: http::StatusCode,
        chain_id: String,
    ) {
        self.provider_status_code_counter
            .add(&otel::Context::new(), 1, &[
                otel::KeyValue::new("provider", provider.provider_kind().to_string()),
                otel::KeyValue::new("status_code", format!("{}", status.as_u16())),
                otel::KeyValue::new("chain_id", chain_id),
            ])
    }

    pub fn record_provider_weight(&self, provider: &ProviderKind, chain_id: String, weight: u64) {
        self.weights_value_recorder
            .record(&otel::Context::new(), weight, &[
                otel::KeyValue::new("provider", provider.to_string()),
                otel::KeyValue::new("chain_id", chain_id),
            ])
    }

    pub fn add_identity_lookup(&self) {
        self.identity_lookup_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_identity_lookup_success(&self, source: &IdentityLookupSource) {
        self.identity_lookup_success_counter
            .add(&otel::Context::new(), 1, &[otel::KeyValue::new(
                "source",
                source.as_str(),
            )]);
    }

    pub fn add_identity_lookup_latency(&self, latency: Duration, source: &IdentityLookupSource) {
        self.identity_lookup_latency_tracker.record(
            &otel::Context::new(),
            latency.as_secs_f64(),
            &[otel::KeyValue::new("source", source.as_str())],
        );
    }

    pub fn add_identity_lookup_cache_latency(&self, start: SystemTime) {
        self.identity_lookup_cache_latency_tracker.record(
            &otel::Context::new(),
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_identity_lookup_name(&self) {
        self.identity_lookup_name_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_identity_lookup_name_success(&self) {
        self.identity_lookup_name_success_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_identity_lookup_name_latency(&self, start: SystemTime) {
        self.identity_lookup_name_latency_tracker.record(
            &otel::Context::new(),
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_identity_lookup_avatar(&self) {
        self.identity_lookup_avatar_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_identity_lookup_avatar_success(&self) {
        self.identity_lookup_avatar_success_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_identity_lookup_avatar_latency(&self, start: SystemTime) {
        self.identity_lookup_avatar_latency_tracker.record(
            &otel::Context::new(),
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_identity_lookup_name_present(&self) {
        self.identity_lookup_name_present_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_identity_lookup_avatar_present(&self) {
        self.identity_lookup_avatar_present_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_websocket_connection(&self, chain_id: String) {
        self.websocket_connection_counter
            .add(&otel::Context::new(), 1, &[otel::KeyValue::new(
                "chain_id", chain_id,
            )]);
    }

    pub fn add_history_lookup(&self, provider: &ProviderKind) {
        self.history_lookup_counter
            .add(&otel::Context::new(), 1, &[otel::KeyValue::new(
                "provider",
                provider.to_string(),
            )]);
    }

    pub fn add_history_lookup_success(&self, provider: &ProviderKind) {
        self.history_lookup_success_counter
            .add(&otel::Context::new(), 1, &[otel::KeyValue::new(
                "provider",
                provider.to_string(),
            )]);
    }

    pub fn add_history_lookup_latency(&self, provider: &ProviderKind, latency: Duration) {
        self.history_lookup_latency_tracker.record(
            &otel::Context::new(),
            latency.as_secs_f64(),
            &[otel::KeyValue::new("provider", provider.to_string())],
        );
    }

    fn add_cpu_usage(&self, usage: f64, cpu_id: f64) {
        self.cpu_usage
            .record(&otel::Context::new(), usage, &[otel::KeyValue::new(
                "cpu", cpu_id,
            )]);
    }

    fn add_memory_total(&self, memory: f64) {
        self.memory_total.record(&otel::Context::new(), memory, &[]);
    }

    fn add_memory_used(&self, memory: f64) {
        self.memory_used.record(&otel::Context::new(), memory, &[]);
    }

    pub fn add_rate_limited_entries_count(&self, entry: u64) {
        self.rate_limited_entries_counter
            .record(&otel::Context::new(), entry, &[]);
    }

    /// Gathering system CPU(s) and Memory usage metrics
    pub async fn gather_system_metrics(&self) {
        let mut system = System::new_with_specifics(
            RefreshKind::new()
                .with_memory(MemoryRefreshKind::new().with_ram())
                .with_cpu(CpuRefreshKind::everything().without_frequency()),
        );
        system.refresh_all();

        // Wait a bit because CPU usage is based on diff.
        // https://docs.rs/sysinfo/0.30.5/sysinfo/struct.Cpu.html#method.cpu_usage
        tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;
        system.refresh_cpu();

        for (i, processor) in system.cpus().iter().enumerate() {
            self.add_cpu_usage(processor.cpu_usage() as f64, i as f64);
        }

        self.add_memory_total(system.total_memory() as f64);
        self.add_memory_used(system.used_memory() as f64);
    }
}
