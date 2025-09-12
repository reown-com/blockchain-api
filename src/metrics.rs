use {
    crate::{
        database::helpers::get_account_names_stats,
        handlers::identity::IdentityLookupSource,
        providers::{ProviderKind, RpcProvider},
        storage::irn::OperationType,
        utils::crypto::CaipNamespaces,
    },
    sqlx::PgPool,
    std::time::{Duration, Instant, SystemTime},
    sysinfo::{
        CpuRefreshKind, MemoryRefreshKind, RefreshKind, System, MINIMUM_CPU_UPDATE_INTERVAL,
    },
    tracing::{error, instrument},
    wc::metrics::{counter, gauge, histogram, EnumLabel, StringLabel},
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
pub struct Metrics {}

impl Metrics {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Metrics {}
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
        histogram!("rpc_call_retries", StringLabel<"chain_id", String> => &chain_id)
            .record(retires_count as f64);
    }

    pub fn add_rpc_cached_call(&self, chain_id: String, method: String) {
        counter!("rpc_cached_call_counter", 
            StringLabel<"chain_id", String> => &chain_id, 
            StringLabel<"method", String> => &method)
        .increment(1);
    }

    pub fn add_balance_lookup_retries(&self, retry_count: u64, namespace: CaipNamespaces) {
        histogram!("balance_lookup_retries", 
            StringLabel<"namespace", String> => &namespace.to_string())
        .record(retry_count as f64);
    }

    pub fn add_http_call(&self, code: u16, route: String) {
        counter!("http_call_counter", 
            StringLabel<"code", String> => &code.to_string(), 
            StringLabel<"route", String> => &route)
        .increment(1);
    }

    pub fn add_http_latency(&self, code: u16, route: String, latency: f64) {
        histogram!("http_latency_tracker",
            StringLabel<"code", String> => &code.to_string(),
            StringLabel<"route", String> => &route
        )
        .record(latency);
    }

    pub fn add_external_http_latency(
        &self,
        provider_kind: &ProviderKind,
        start: SystemTime,
        chain_id: Option<String>,
        endpoint: Option<String>,
    ) {
        histogram!("http_external_latency_tracker", 
            StringLabel<"provider", String> => &provider_kind.to_string(), 
            StringLabel<"chain_id", String> => &chain_id.unwrap_or_default(), 
            StringLabel<"endpoint", String> => &endpoint.unwrap_or_default())
        .record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
        );
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

    pub fn add_exchange_reconciler_fetch_batch_latency(&self, start: Instant) {
        histogram!("exchange_reconciler_fetch_batch_latency").record(start.elapsed().as_secs_f64());
    }

    pub fn add_exchange_reconciler_process_batch_latency(&self, start: Instant) {
        histogram!("exchange_reconciler_process_batch_latency")
            .record(start.elapsed().as_secs_f64());
    }

    pub fn record_provider_weight(&self, provider: &ProviderKind, chain_id: String, weight: u64) {
        gauge!("provider_weights",
            StringLabel<"provider", String> => &provider.to_string(),
            StringLabel<"chain_id", String> => &chain_id
        )
        .set(weight as f64);
    }

    pub fn add_no_providers_for_chain(&self, chain_id: String) {
        counter!("no_providers_for_chain_counter",
            StringLabel<"chain_id", String> => &chain_id
        )
        .increment(1);
    }

    pub fn add_found_provider_for_chain(&self, chain_id: String, provider_kind: &ProviderKind) {
        counter!("found_provider_for_chain_counter",
            StringLabel<"chain_id", String> => &chain_id,
            StringLabel<"provider", String> => &provider_kind.to_string()
        )
        .increment(1);
    }

    pub fn add_chain_latency(
        &self,
        provider_kind: &ProviderKind,
        start: SystemTime,
        chain_id: String,
    ) {
        histogram!("chain_latency_tracker",
            StringLabel<"provider", String> => &provider_kind.to_string(),
            StringLabel<"chain_id", String> => &chain_id
        )
        .record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
        );
    }

    pub fn add_identity_lookup(&self) {
        counter!("identity_lookup_counter").increment(1);
    }

    pub fn add_identity_lookup_success(&self, source: &IdentityLookupSource) {
        counter!("identity_lookup_success_counter", EnumLabel<"source", IdentityLookupSource> => *source)
            .increment(1);
    }

    pub fn add_identity_lookup_latency(&self, latency: Duration, source: &IdentityLookupSource) {
        histogram!("identity_lookup_latency_tracker", EnumLabel<"source", IdentityLookupSource> => *source)
            .record(latency.as_secs_f64());
    }

    pub fn add_identity_lookup_cache_latency(&self, start: SystemTime) {
        histogram!("identity_lookup_cache_latency_tracker").record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
        );
    }

    pub fn add_identity_lookup_name(&self) {
        counter!("identity_lookup_name_counter").increment(1);
    }

    pub fn add_identity_lookup_name_success(&self) {
        counter!("identity_lookup_name_success_counter").increment(1);
    }

    pub fn add_identity_lookup_name_latency(&self, start: SystemTime) {
        histogram!("identity_lookup_name_latency_tracker").record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
        );
    }

    pub fn add_identity_lookup_avatar(&self) {
        counter!("identity_lookup_avatar_counter").increment(1);
    }

    pub fn add_identity_lookup_avatar_success(&self) {
        counter!("identity_lookup_avatar_success_counter").increment(1);
    }

    pub fn add_identity_lookup_avatar_latency(&self, start: SystemTime) {
        histogram!("identity_lookup_avatar_latency_tracker").record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
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
        histogram!("history_lookup_latency_tracker", StringLabel<"provider", String> => &provider.to_string())
            .record(latency.as_secs_f64());
    }

    fn add_cpu_usage(&self, usage: f64, cpu_id: f64) {
        histogram!("cpu_usage", StringLabel<"cpu", String> => &cpu_id.to_string()).record(usage);
    }

    fn add_memory_total(&self, memory: f64) {
        histogram!("memory_total").record(memory);
    }

    fn add_memory_used(&self, memory: f64) {
        histogram!("memory_used").record(memory);
    }

    pub fn add_rate_limited_entries_count(&self, entry: u64) {
        histogram!("rate_limited_entries_counter").record(entry as f64);
    }

    pub fn add_rate_limiting_latency(&self, start: SystemTime) {
        histogram!("rate_limiting_latency_tracker").record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
        );
    }

    pub fn add_non_rpc_providers_cache_latency(&self, start: SystemTime) {
        histogram!("non_rpc_providers_cache_latency_tracker").record(
            start
                .elapsed()
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
        );
    }

    pub fn add_rate_limited_response(&self) {
        counter!("rate_limited_responses_counter").increment(1);
    }

    pub fn add_irn_latency(&self, start: SystemTime, operation: OperationType) {
        histogram!("irn_latency_tracker", EnumLabel<"operation", OperationType> => operation)
            .record(
                start
                    .elapsed()
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs_f64(),
            );
    }

    pub fn add_ca_gas_estimation(
        &self,
        gas: u64,
        chain_id: String,
        tx_type: ChainAbstractionTransactionType,
    ) {
        histogram!("ca_gas_estimation_tracker",
            StringLabel<"chain_id", String> => &chain_id,
            StringLabel<"tx_type", String> => &tx_type.to_string()
        )
        .record(gas as f64);
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
                gauge!("account_names_count").set(names_stats.count as f64);
            }
            Err(e) => {
                error!("Error on getting account names stats from database: {e:?}");
            }
        }
    }
}
