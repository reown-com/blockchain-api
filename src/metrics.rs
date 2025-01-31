use {
    crate::{
        database::helpers::get_account_names_stats,
        handlers::identity::IdentityLookupSource,
        providers::{ProviderKind, RpcProvider},
        storage::irn::OperationType,
    },
    sqlx::PgPool,
    std::time::{Duration, SystemTime},
    sysinfo::{
        CpuRefreshKind, MemoryRefreshKind, RefreshKind, System, MINIMUM_CPU_UPDATE_INTERVAL,
    },
    tracing::{error, instrument},
    wc::metrics::{
        otel::{
            self,
            metrics::{Counter, Histogram, ObservableGauge},
        },
        ServiceMetrics,
    },
};

#[derive(strum_macros::Display)]
pub enum ChainAbstractionTransactionType {
    Transfer,
    Approve,
    Bridge,
}

#[derive(strum_macros::Display)]
pub enum ChainAbstractionNoBridgingNeededType {
    NativeTokenTransfer,
    AssetNotSupported,
    SufficientFunds,
}

#[derive(Debug)]
pub struct Metrics {
    pub rpc_call_counter: Counter<u64>,
    pub rpc_call_retries: Histogram<u64>,
    pub http_call_counter: Counter<u64>,
    pub provider_finished_call_counter: Counter<u64>,
    pub provider_failed_call_counter: Counter<u64>,
    pub no_providers_for_chain_counter: Counter<u64>,
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

    // Generic Non-RPC providers caching
    pub non_rpc_providers_cache_latency_tracker: Histogram<f64>,

    // System metrics
    pub cpu_usage: Histogram<f64>,
    pub memory_total: Histogram<f64>,
    pub memory_used: Histogram<f64>,

    // Rate limiting
    pub rate_limited_entries_counter: Histogram<u64>,
    pub rate_limiting_latency_tracker: Histogram<f64>,
    pub rate_limited_responses_counter: Counter<u64>,

    // Account names
    pub account_names_count: ObservableGauge<u64>,

    // IRN client
    pub irn_latency_tracker: Histogram<f64>,

    // Chain Abstracton
    pub ca_gas_estimation_tracker: Histogram<f64>,
    pub ca_no_routes_found_counter: Counter<u64>,
    pub ca_insufficient_funds_counter: Counter<u64>,
    pub ca_no_bridging_needed_counter: Counter<u64>,
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

        let account_names_count = meter
            .u64_observable_gauge("account_names_count")
            .with_description("Registered account names count")
            .init();

        let irn_latency_tracker = meter
            .f64_histogram("irn_latency_tracker")
            .with_description("The latency of IRN client calls")
            .init();

        let rate_limiting_latency_tracker = meter
            .f64_histogram("rate_limiting_latency_tracker")
            .with_description("Rate limiting latency tracker")
            .init();

        let rate_limited_responses_counter = meter
            .u64_counter("rate_limited_responses_counter")
            .with_description("Rate limiting responses counter")
            .init();
        let non_rpc_providers_cache_latency_tracker = meter
            .f64_histogram("non_rpc_providers_cache_latency_tracker")
            .with_description("The latency of non-RPC providers cache lookups")
            .init();

        let no_providers_for_chain_counter = meter
            .u64_counter("no_providers_for_chain_counter")
            .with_description("The number of chain RPC calls that had no available providers")
            .init();

        let ca_gas_estimation_tracker = meter
            .f64_histogram("gas_estimation")
            .with_description("The gas estimation for transactions")
            .init();

        let ca_no_routes_found_counter = meter
            .u64_counter("ca_no_routes_found")
            .with_description("The number of times no routes were found for a CA")
            .init();

        let ca_insufficient_funds_counter = meter
            .u64_counter("ca_insufficient_funds")
            .with_description("The number of times insufficient funds were responded for a CA")
            .init();

        let ca_no_bridging_needed_counter = meter
            .u64_counter("ca_no_bridging_needed")
            .with_description("The number of times no bridging was needed for a CA")
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
            no_providers_for_chain_counter,
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
            non_rpc_providers_cache_latency_tracker,
            cpu_usage,
            memory_total,
            memory_used,
            account_names_count,
            irn_latency_tracker,
            rate_limiting_latency_tracker,
            rate_limited_entries_counter,
            rate_limited_responses_counter,
            ca_gas_estimation_tracker,
            ca_no_routes_found_counter,
            ca_insufficient_funds_counter,
            ca_no_bridging_needed_counter,
        }
    }
}

impl Metrics {
    pub fn add_rpc_call(&self, chain_id: String) {
        self.rpc_call_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new("chain.id", chain_id)],
        );
    }

    pub fn add_rpc_call_retries(&self, retires_count: u64, chain_id: String) {
        self.rpc_call_retries.record(
            &otel::Context::new(),
            retires_count,
            &[otel::KeyValue::new("chain_id", chain_id)],
        )
    }

    pub fn add_http_call(&self, code: u16, route: String) {
        self.http_call_counter.add(
            &otel::Context::new(),
            1,
            &[
                otel::KeyValue::new("code", i64::from(code)),
                otel::KeyValue::new("route", route),
            ],
        );
    }

    pub fn add_http_latency(&self, code: u16, route: String, latency: f64) {
        self.http_latency_tracker.record(
            &otel::Context::new(),
            latency,
            &[
                otel::KeyValue::new("code", i64::from(code)),
                otel::KeyValue::new("route", route),
            ],
        )
    }

    pub fn add_external_http_latency(
        &self,
        provider_kind: ProviderKind,
        start: SystemTime,
        endpoint: Option<String>,
    ) {
        let mut attributes = vec![otel::KeyValue::new("provider", provider_kind.to_string())];
        if let Some(endpoint) = endpoint {
            attributes.push(otel::KeyValue::new("endpoint", endpoint));
        }

        self.http_external_latency_tracker.record(
            &otel::Context::new(),
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &attributes,
        )
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
        self.rate_limited_call_counter.add(
            &otel::Context::new(),
            1,
            &[
                otel::KeyValue::new("provider_kind", provider.provider_kind().to_string()),
                otel::KeyValue::new("project_id", project_id),
            ],
        )
    }

    pub fn add_failed_provider_call(&self, provider: &dyn RpcProvider) {
        self.provider_failed_call_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new(
                "provider",
                provider.provider_kind().to_string(),
            )],
        )
    }

    pub fn add_finished_provider_call(&self, provider: &dyn RpcProvider) {
        self.provider_finished_call_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new(
                "provider",
                provider.provider_kind().to_string(),
            )],
        )
    }

    pub fn add_status_code_for_provider(
        &self,
        provider_kind: ProviderKind,
        status: u16,
        chain_id: Option<String>,
        endpoint: Option<String>,
    ) {
        let mut attributes = vec![
            otel::KeyValue::new("provider", provider_kind.to_string()),
            otel::KeyValue::new("status_code", format!("{}", status)),
        ];
        if let Some(chain_id) = chain_id {
            attributes.push(otel::KeyValue::new("chain_id", chain_id));
        }
        if let Some(endpoint) = endpoint {
            attributes.push(otel::KeyValue::new("endpoint", endpoint));
        }

        self.provider_status_code_counter
            .add(&otel::Context::new(), 1, &attributes)
    }

    pub fn add_latency_and_status_code_for_provider(
        &self,
        provider_kind: ProviderKind,
        status: u16,
        latency: SystemTime,
        chain_id: Option<String>,
        endpoint: Option<String>,
    ) {
        self.add_status_code_for_provider(provider_kind, status, chain_id, endpoint.clone());
        self.add_external_http_latency(provider_kind, latency, endpoint);
    }

    pub fn record_provider_weight(&self, provider: &ProviderKind, chain_id: String, weight: u64) {
        self.weights_value_recorder.record(
            &otel::Context::new(),
            weight,
            &[
                otel::KeyValue::new("provider", provider.to_string()),
                otel::KeyValue::new("chain_id", chain_id),
            ],
        )
    }

    pub fn add_no_providers_for_chain(&self, chain_id: String) {
        self.no_providers_for_chain_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new("chain_id", chain_id)],
        );
    }

    pub fn add_identity_lookup(&self) {
        self.identity_lookup_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_identity_lookup_success(&self, source: &IdentityLookupSource) {
        self.identity_lookup_success_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new("source", source.as_str())],
        );
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
        self.websocket_connection_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new("chain_id", chain_id)],
        );
    }

    pub fn add_history_lookup(&self, provider: &ProviderKind) {
        self.history_lookup_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new("provider", provider.to_string())],
        );
    }

    pub fn add_history_lookup_success(&self, provider: &ProviderKind) {
        self.history_lookup_success_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new("provider", provider.to_string())],
        );
    }

    pub fn add_history_lookup_latency(&self, provider: &ProviderKind, latency: Duration) {
        self.history_lookup_latency_tracker.record(
            &otel::Context::new(),
            latency.as_secs_f64(),
            &[otel::KeyValue::new("provider", provider.to_string())],
        );
    }

    fn add_cpu_usage(&self, usage: f64, cpu_id: f64) {
        self.cpu_usage.record(
            &otel::Context::new(),
            usage,
            &[otel::KeyValue::new("cpu", cpu_id)],
        );
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

    pub fn add_rate_limiting_latency(&self, start: SystemTime) {
        self.rate_limiting_latency_tracker.record(
            &otel::Context::new(),
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_non_rpc_providers_cache_latency(&self, start: SystemTime) {
        self.non_rpc_providers_cache_latency_tracker.record(
            &otel::Context::new(),
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_rate_limited_response(&self) {
        self.rate_limited_responses_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_irn_latency(&self, start: SystemTime, operation: OperationType) {
        self.irn_latency_tracker.record(
            &otel::Context::new(),
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[otel::KeyValue::new("operation", operation.as_str())],
        );
    }

    pub fn add_ca_gas_estimation(
        &self,
        gas: u64,
        chain_id: String,
        tx_type: ChainAbstractionTransactionType,
    ) {
        self.ca_gas_estimation_tracker.record(
            &otel::Context::new(),
            gas as f64,
            &[
                otel::KeyValue::new("chain_id", chain_id),
                otel::KeyValue::new("tx_type", tx_type.to_string()),
            ],
        );
    }

    pub fn add_ca_no_routes_found(&self, route: String) {
        self.ca_no_routes_found_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new("route", route)],
        );
    }

    pub fn add_ca_insufficient_funds(&self) {
        self.ca_insufficient_funds_counter
            .add(&otel::Context::new(), 1, &[]);
    }

    pub fn add_ca_no_bridging_needed(&self, ca_type: ChainAbstractionNoBridgingNeededType) {
        self.ca_no_bridging_needed_counter.add(
            &otel::Context::new(),
            1,
            &[otel::KeyValue::new("type", ca_type.to_string())],
        );
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

    /// Update the account names count from database
    #[instrument(skip_all, level = "debug")]
    pub async fn update_account_names_count(&self, postgres: &PgPool) {
        let names_stats = get_account_names_stats(postgres).await;
        match names_stats {
            Ok(names_stats) => {
                self.account_names_count.observe(
                    &otel::Context::new(),
                    names_stats.count as u64,
                    &[],
                );
            }
            Err(e) => {
                error!(
                    "Error on getting account names stats from database: {:?}",
                    e
                );
            }
        }
    }
}
