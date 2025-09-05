use {
    crate::{
        database::helpers::get_account_names_stats,
        handlers::identity::IdentityLookupSource,
        providers::{ProviderKind, RpcProvider},
        storage::irn::OperationType,
        utils::crypto::CaipNamespaces,
    },
    sqlx::PgPool,
    std::time::{Duration, SystemTime},
    sysinfo::{
        CpuRefreshKind, MemoryRefreshKind, RefreshKind, System, MINIMUM_CPU_UPDATE_INTERVAL,
    },
    tracing::{error, instrument},
    wc::metrics::{counter, Histogram, ObservableGauge, ServiceMetrics, StringLabel},
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
    pub rpc_call_retries: Histogram,
    pub chain_latency_tracker: Histogram, // Chain latency
    pub http_latency_tracker: Histogram,
    pub http_external_latency_tracker: Histogram,
    pub weights_value_recorder: Histogram,
    pub identity_lookup_latency_tracker: Histogram,
    pub identity_lookup_cache_latency_tracker: Histogram,
    pub identity_lookup_name_latency_tracker: Histogram,
    pub identity_lookup_avatar_latency_tracker: Histogram,
    pub history_lookup_latency_tracker: Histogram,
    pub balance_lookup_retries: Histogram,

    // Generic Non-RPC providers caching
    pub non_rpc_providers_cache_latency_tracker: Histogram,

    // System metrics
    pub cpu_usage: Histogram,
    pub memory_total: Histogram,
    pub memory_used: Histogram,

    // Rate limiting
    pub rate_limited_entries_counter: Histogram,
    pub rate_limiting_latency_tracker: Histogram,

    // Account names
    pub account_names_count: ObservableGauge<u64>,

    // IRN client
    pub irn_latency_tracker: Histogram,

    // Chain Abstracton
    pub ca_gas_estimation_tracker: Histogram,
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

        let rpc_cached_call_counter = meter
            .u64_counter("rpc_cached_call_counter")
            .with_description("The number of cached rpc calls served")
            .init();

        let chain_latency_tracker = meter
            .f64_histogram("chain_latency_tracker")
            .with_description("The chain latency")
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

        let provider_internal_error_code_counter = meter
            .u64_counter("provider_internal_error_code_counter")
            .with_description(
                "The count of JSON-RPC internal error codes returned by providers response",
            )
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

        let found_provider_for_chain_counter = meter
            .u64_counter("found_provider_for_chain_counter")
            .with_description(
                "The number of chain RPC calls that had at least one available provider",
            )
            .init();

        let ca_gas_estimation_tracker = meter
            .f64_histogram("gas_estimation")
            .with_description("The gas estimation for transactions")
            .init();

        let ca_no_routes_found_counter = meter
            .u64_counter("ca_no_routes_found")
            .with_description("The number of times no routes were found for a CA")
            .init();

        let ca_routes_found_counter = meter
            .u64_counter("ca_routes_found")
            .with_description("The number of times of sucess routes were found for a CA")
            .init();

        let ca_insufficient_funds_counter = meter
            .u64_counter("ca_insufficient_funds")
            .with_description("The number of times insufficient funds were responded for a CA")
            .init();

        let ca_no_bridging_needed_counter = meter
            .u64_counter("ca_no_bridging_needed")
            .with_description("The number of times no bridging was needed for a CA")
            .init();

        let balance_lookup_retries = meter
            .u64_histogram("balance_lookup_retries")
            .with_description("Retries per balance call")
            .init();

        Metrics {
            rpc_call_retries,
            chain_latency_tracker,
            http_external_latency_tracker,
            http_latency_tracker,
            weights_value_recorder,
            identity_lookup_latency_tracker,
            identity_lookup_cache_latency_tracker,
            identity_lookup_name_latency_tracker,
            identity_lookup_avatar_latency_tracker,
            history_lookup_latency_tracker,
            balance_lookup_retries,
            non_rpc_providers_cache_latency_tracker,
            cpu_usage,
            memory_total,
            memory_used,
            account_names_count,
            irn_latency_tracker,
            rate_limiting_latency_tracker,
            ca_gas_estimation_tracker,
        }
    }
}

impl Metrics {
    pub fn add_rpc_call(&self, chain_id: String, provider_kind: &ProviderKind) {
        counter!("rpc_call_counter", 
            StringLabel<"chain_id", String> => &chain_id, 
            StringLabel<"provider", String> => &provider_kind.to_string())
        .increment(1);
    }

    pub fn add_rpc_call_retries(&self, retires_count: u64, chain_id: String) {
        counter!("rpc_call_retries", StringLabel<"chain_id", String> => &chain_id)
            .increment(retires_count);
    }

    pub fn add_rpc_cached_call(&self, chain_id: String, method: String) {
        counter!("rpc_cached_call_counter", 
            StringLabel<"chain_id", String> => &chain_id, 
            StringLabel<"method", String> => &method)
        .increment(1);
    }

    pub fn add_balance_lookup_retries(&self, retry_count: u64, namespace: CaipNamespaces) {
        counter!("balance_lookup_retries", 
            StringLabel<"namespace", String> => &namespace.to_string())
        .increment(retry_count);
    }

    pub fn add_http_call(&self, code: u16, route: String) {
        counter!("http_call_counter", 
            StringLabel<"code", String> => &code.to_string(), 
            StringLabel<"route", String> => &route)
        .increment(1);
    }

    pub fn add_http_latency(&self, code: u16, route: String, latency: f64) {
        self.http_latency_tracker.record(
            latency,
            &[
                otel::KeyValue::new("code", i64::from(code)),
                otel::KeyValue::new("route", route),
            ],
        )
    }

    pub fn add_external_http_latency(
        &self,
        provider_kind: &ProviderKind,
        start: SystemTime,
        chain_id: Option<String>,
        endpoint: Option<String>,
    ) {
        let mut attributes = vec![otel::KeyValue::new("provider", provider_kind.to_string())];
        if let Some(chain_id) = chain_id {
            attributes.push(otel::KeyValue::new("chain_id", chain_id));
        }
        if let Some(endpoint) = endpoint {
            attributes.push(otel::KeyValue::new("endpoint", endpoint));
        }

        self.http_external_latency_tracker.record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &attributes,
        )
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn add_rejected_project(&self) {
        counter!("rejected_project_counter").increment(1);
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn add_quota_limited_project(&self) {
        counter!("quota_limited_project_counter").increment(1);
    }

    pub fn add_rate_limited_call(&self, provider: &dyn RpcProvider, project_id: String) {
        counter!("rate_limited_call_counter", 
            StringLabel<"provider_kind", String> => &provider.provider_kind().to_string(), 
            StringLabel<"project_id", String> => &project_id)
        .increment(1);
    }

    pub fn add_failed_provider_call(&self, chain_id: String, provider: &dyn RpcProvider) {
        counter!("provider_failed_call_counter", 
            StringLabel<"chain_id", String> => &chain_id, 
            StringLabel<"provider", String> => &provider.provider_kind().to_string())
        .increment(1);
    }

    pub fn add_finished_provider_call(&self, chain_id: String, provider: &dyn RpcProvider) {
        counter!("provider_finished_call_counter", 
            StringLabel<"chain_id", String> => &chain_id, 
            StringLabel<"provider", String> => &provider.provider_kind().to_string())
        .increment(1);
    }

    pub fn add_status_code_for_provider(
        &self,
        provider_kind: &ProviderKind,
        status: u16,
        chain_id: Option<String>,
        endpoint: Option<String>,
    ) {
        counter!("provider_status_code_counter", 
            StringLabel<"provider", String> => &provider_kind.to_string(), 
            StringLabel<"status_code", String> => &status.to_string(), 
            StringLabel<"chain_id", String> => &chain_id.unwrap_or_default(), 
            StringLabel<"endpoint", String> => &endpoint.unwrap_or_default())
        .increment(1);
    }

    pub fn add_internal_error_code_for_provider(
        &self,
        provider_kind: ProviderKind,
        chain_id: String,
        code: i32,
    ) {
        counter!("provider_internal_error_code_counter", 
            StringLabel<"provider", String> => &provider_kind.to_string(), 
            StringLabel<"chain_id", String> => &chain_id, 
            StringLabel<"code", String> => &code.to_string())
        .increment(1);
    }

    pub fn add_latency_and_status_code_for_provider(
        &self,
        provider_kind: &ProviderKind,
        status: u16,
        start: SystemTime,
        chain_id: Option<String>,
        endpoint: Option<String>,
    ) {
        self.add_status_code_for_provider(
            provider_kind,
            status,
            chain_id.clone(),
            endpoint.clone(),
        );
        self.add_external_http_latency(provider_kind, start, chain_id, endpoint);
    }

    pub fn record_provider_weight(&self, provider: &ProviderKind, chain_id: String, weight: u64) {
        self.weights_value_recorder.record(
            weight,
            &[
                otel::KeyValue::new("provider", provider.to_string()),
                otel::KeyValue::new("chain_id", chain_id),
            ],
        )
    }

    pub fn add_no_providers_for_chain(&self, chain_id: String) {
        counter!("no_providers_for_chain_counter", StringLabel<"chain_id", String> => &chain_id)
            .increment(1);
    }

    pub fn add_found_provider_for_chain(&self, chain_id: String, provider_kind: &ProviderKind) {
        counter!("found_provider_for_chain_counter", StringLabel<"chain_id", String> => &chain_id, StringLabel<"provider", String> => &provider_kind.to_string())
            .increment(1);
    }

    pub fn add_chain_latency(
        &self,
        provider_kind: &ProviderKind,
        start: SystemTime,
        chain_id: String,
    ) {
        self.chain_latency_tracker.record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[
                otel::KeyValue::new("provider", provider_kind.to_string()),
                otel::KeyValue::new("chain_id", chain_id),
            ],
        )
    }

    pub fn add_identity_lookup(&self) {
        counter!("identity_lookup_counter").increment(1);
    }

    pub fn add_identity_lookup_success(&self, source: &IdentityLookupSource) {
        counter!("identity_lookup_success_counter", StringLabel<"source", String> => &source.as_str())
            .increment(1);
    }

    pub fn add_identity_lookup_latency(&self, latency: Duration, source: &IdentityLookupSource) {
        self.identity_lookup_latency_tracker.record(
            latency.as_secs_f64(),
            &[otel::KeyValue::new("source", source.as_str())],
        );
    }

    pub fn add_identity_lookup_cache_latency(&self, start: SystemTime) {
        self.identity_lookup_cache_latency_tracker.record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_identity_lookup_name(&self) {
        counter!("identity_lookup_name_counter").increment(1);
    }

    pub fn add_identity_lookup_name_success(&self) {
        counter!("identity_lookup_name_success_counter").increment(1);
    }

    pub fn add_identity_lookup_name_latency(&self, start: SystemTime) {
        self.identity_lookup_name_latency_tracker.record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_identity_lookup_avatar(&self) {
        counter!("identity_lookup_avatar_counter").increment(1);
    }

    pub fn add_identity_lookup_avatar_success(&self) {
        counter!("identity_lookup_avatar_success_counter").increment(1);
    }

    pub fn add_identity_lookup_avatar_latency(&self, start: SystemTime) {
        self.identity_lookup_avatar_latency_tracker.record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_identity_lookup_name_present(&self) {
        counter!("identity_lookup_name_present_counter").increment(1);
    }

    pub fn add_identity_lookup_avatar_present(&self) {
        counter!("identity_lookup_avatar_present_counter").increment(1);
    }

    pub fn add_websocket_connection(&self, chain_id: String) {
        counter!("websocket_connection_counter", StringLabel<"chain_id", String> => &chain_id)
            .increment(1);
    }

    pub fn add_history_lookup(&self, provider: &ProviderKind) {
        counter!("history_lookup_counter", StringLabel<"provider", String> => &provider.to_string())
            .increment(1);
    }

    pub fn add_history_lookup_success(&self, provider: &ProviderKind) {
        counter!("history_lookup_success_counter", StringLabel<"provider", String> => &provider.to_string())
            .increment(1);
    }

    pub fn add_history_lookup_latency(&self, provider: &ProviderKind, latency: Duration) {
        self.history_lookup_latency_tracker.record(
            latency.as_secs_f64(),
            &[otel::KeyValue::new("provider", provider.to_string())],
        );
    }

    fn add_cpu_usage(&self, usage: f64, cpu_id: f64) {
        self.cpu_usage
            .record(usage, &[otel::KeyValue::new("cpu", cpu_id)]);
    }

    fn add_memory_total(&self, memory: f64) {
        self.memory_total.record(memory, &[]);
    }

    fn add_memory_used(&self, memory: f64) {
        self.memory_used.record(memory, &[]);
    }

    pub fn add_rate_limited_entries_count(&self, entry: u64) {
        self.rate_limited_entries_counter.record(entry, &[]);
    }

    pub fn add_rate_limiting_latency(&self, start: SystemTime) {
        self.rate_limiting_latency_tracker.record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_non_rpc_providers_cache_latency(&self, start: SystemTime) {
        self.non_rpc_providers_cache_latency_tracker.record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
            &[],
        );
    }

    pub fn add_rate_limited_response(&self) {
        counter!("rate_limited_responses_counter").increment(1);
    }

    pub fn add_irn_latency(&self, start: SystemTime, operation: OperationType) {
        self.irn_latency_tracker.record(
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
            gas as f64,
            &[
                otel::KeyValue::new("chain_id", chain_id),
                otel::KeyValue::new("tx_type", tx_type.to_string()),
            ],
        );
    }

    pub fn add_ca_no_routes_found(&self, route: String) {
        counter!("ca_no_routes_found_counter", StringLabel<"route", String> => &route).increment(1);
    }

    pub fn add_ca_routes_found(&self, route: String) {
        counter!("ca_routes_found_counter", StringLabel<"route", String> => &route).increment(1);
    }

    pub fn add_ca_insufficient_funds(&self) {
        counter!("ca_insufficient_funds_counter").increment(1);
    }

    pub fn add_ca_no_bridging_needed(&self, ca_type: ChainAbstractionNoBridgingNeededType) {
        counter!("ca_no_bridging_needed_counter", StringLabel<"type", String> => &ca_type.to_string())
            .increment(1);
    }

    /// Gathering system CPU(s) and Memory usage metrics
    pub async fn gather_system_metrics(&self) {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_memory(MemoryRefreshKind::everything().with_ram())
                .with_cpu(CpuRefreshKind::everything().without_frequency()),
        );
        system.refresh_all();

        // Wait a bit because CPU usage is based on diff.
        // https://docs.rs/sysinfo/0.30.5/sysinfo/struct.Cpu.html#method.cpu_usage
        tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;
        system.refresh_cpu_all();

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
                self.account_names_count
                    .observe(names_stats.count as u64, &[]);
            }
            Err(e) => {
                error!("Error on getting account names stats from database: {e:?}");
            }
        }
    }
}
