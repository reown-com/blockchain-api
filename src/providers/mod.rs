use {
    self::zerion::ZerionProvider,
    crate::{
        env::ProviderConfig,
        error::RpcError,
        handlers::{HistoryQueryParams, HistoryResponseBody},
    },
    axum::response::Response,
    axum_tungstenite::WebSocketUpgrade,
    hyper::http::HeaderValue,
    rand::{distributions::WeightedIndex, prelude::Distribution, rngs::OsRng},
    std::{fmt::Debug, hash::Hash, sync::Arc},
    tracing::{info, log::warn},
    wc::metrics::TaskMetrics,
};

mod base;
mod binance;
mod infura;
mod omnia;
mod pokt;
mod publicnode;
mod weights;
mod zerion;
mod zksync;
mod zora;

use {
    crate::{error::RpcResult, handlers::RpcQueryParams},
    async_trait::async_trait,
    std::{collections::HashMap, fmt::Display},
};
pub use {
    base::BaseProvider,
    binance::BinanceProvider,
    infura::{InfuraProvider, InfuraWsProvider},
    omnia::OmniatechProvider,
    pokt::PoktProvider,
    publicnode::PublicnodeProvider,
    zksync::ZKSyncProvider,
    zora::{ZoraProvider, ZoraWsProvider},
};

static WS_PROXY_TASK_METRICS: TaskMetrics = TaskMetrics::new("ws_proxy_task");

pub type WeightResolver = HashMap<String, HashMap<ProviderKind, Weight>>;

pub struct ProviderRepository {
    providers: HashMap<ProviderKind, Arc<dyn RpcProvider>>,
    ws_providers: HashMap<ProviderKind, Arc<dyn RpcWsProvider>>,

    weight_resolver: WeightResolver,
    ws_weight_resolver: WeightResolver,

    prometheus_client: prometheus_http_query::Client,
    prometheus_workspace_header: String,

    pub history_provider: Arc<dyn HistoryProvider>,
}

impl ProviderRepository {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let prometheus_client = {
            let prometheus_query_url =
                std::env::var("SIG_PROXY_URL").unwrap_or("http://localhost:8080/".into());

            prometheus_http_query::Client::try_from(prometheus_query_url)
                .expect("Failed to connect to prometheus")
        };

        let prometheus_workspace_header =
            std::env::var("SIG_PROM_WORKSPACE_HEADER").unwrap_or("localhost:9090".into());

        // Don't crash the application if the ZERION_API_KEY is not set
        // TODO: find a better way to handle this
        let zerion_api_key =
            std::env::var("RPC_PROXY_ZERION_API_KEY").unwrap_or("ZERION_KEY_UNDEFINED".into());

        let history_provider = Arc::new(ZerionProvider::new(zerion_api_key));

        Self {
            providers: HashMap::new(),
            ws_providers: HashMap::new(),
            weight_resolver: HashMap::new(),
            ws_weight_resolver: HashMap::new(),
            prometheus_client,
            prometheus_workspace_header,
            history_provider,
        }
    }

    pub fn get_provider_for_chain_id(&self, chain_id: &str) -> Option<Arc<dyn RpcProvider>> {
        let Some(providers) = self.weight_resolver.get(chain_id) else {
            return None;
        };

        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| weight.value()).collect();
        let keys = providers.keys().cloned().collect::<Vec<_>>();
        match WeightedIndex::new(weights) {
            Ok(dist) => {
                let random = dist.sample(&mut OsRng);
                let provider = keys.get(random).unwrap();

                self.providers.get(provider).cloned()
            }
            Err(e) => {
                warn!("Failed to create weighted index: {}", e);
                None
            }
        }
    }

    pub fn get_ws_provider_for_chain_id(&self, chain_id: &str) -> Option<Arc<dyn RpcWsProvider>> {
        let Some(providers) = self.ws_weight_resolver.get(chain_id) else {
            return None;
        };

        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| weight.value()).collect();
        let keys = providers.keys().cloned().collect::<Vec<_>>();
        match WeightedIndex::new(weights) {
            Ok(dist) => {
                let random = dist.sample(&mut OsRng);
                let provider = keys.get(random).unwrap();

                self.ws_providers.get(provider).cloned()
            }
            Err(e) => {
                warn!("Failed to create weighted index: {}", e);
                None
            }
        }
    }

    pub fn add_ws_provider<
        T: RpcProviderFactory<C> + RpcWsProvider + 'static,
        C: ProviderConfig,
    >(
        &mut self,
        provider_config: C,
    ) {
        let ws_provider = T::new(&provider_config);
        let arc_ws_provider = Arc::new(ws_provider);

        self.ws_providers
            .insert(provider_config.provider_kind(), arc_ws_provider);

        let provider_kind = provider_config.provider_kind();
        let supported_ws_chains = provider_config.supported_ws_chains();

        supported_ws_chains
            .into_iter()
            .for_each(|(chain_id, (_, weight))| {
                self.ws_weight_resolver
                    .entry(chain_id)
                    .or_default()
                    .insert(provider_kind, weight);
            });
    }

    pub fn add_provider<T: RpcProviderFactory<C> + RpcProvider + 'static, C: ProviderConfig>(
        &mut self,
        provider_config: C,
    ) {
        let provider = T::new(&provider_config);
        let arc_provider = Arc::new(provider);

        self.providers
            .insert(provider_config.provider_kind(), arc_provider);

        let provider_kind = provider_config.provider_kind();
        let supported_chains = provider_config.supported_chains();

        supported_chains
            .into_iter()
            .for_each(|(chain_id, (_, weight))| {
                self.weight_resolver
                    .entry(chain_id)
                    .or_default()
                    .insert(provider_kind, weight);
            });
        info!("Added provider: {}", provider_kind);
    }

    pub async fn update_weights(&self, metrics: &crate::Metrics) {
        info!("Updating weights");

        let Ok(header_value) = HeaderValue::from_str(&self.prometheus_workspace_header) else {
            warn!(
                "Failed to parse prometheus workspace header from {}",
                self.prometheus_workspace_header
            );
            return;
        };

        match self
            .prometheus_client
            .query("round(increase(provider_status_code_counter_total[3h]))")
            .header("host", header_value)
            .get()
            .await
        {
            Ok(data) => {
                let parsed_weights = weights::parse_weights(data);
                weights::update_values(&self.weight_resolver, parsed_weights);
                weights::record_values(&self.weight_resolver, metrics);
            }
            Err(e) => {
                warn!("Failed to update weights from prometheus: {}", e);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Infura,
    Pokt,
    Binance,
    ZKSync,
    Publicnode,
    Omniatech,
    Base,
    Zora,
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ProviderKind::Infura => "Infura",
            ProviderKind::Pokt => "Pokt",
            ProviderKind::Binance => "Binance",
            ProviderKind::ZKSync => "zkSync",
            ProviderKind::Publicnode => "Publicnode",
            ProviderKind::Omniatech => "Omniatech",
            ProviderKind::Base => "Base",
            ProviderKind::Zora => "Zora",
        })
    }
}

impl ProviderKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Infura" => Some(Self::Infura),
            "Pokt" => Some(Self::Pokt),
            "Binance" => Some(Self::Binance),
            "zkSync" => Some(Self::ZKSync),
            "Publicnode" => Some(Self::Publicnode),
            "Omniatech" => Some(Self::Omniatech),
            "Base" => Some(Self::Base),
            "Zora" => Some(Self::Zora),
            _ => None,
        }
    }
}

#[async_trait]
pub trait RpcProvider: Provider {
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response>;
}

pub trait RpcProviderFactory<T: ProviderConfig>: Provider {
    fn new(provider_config: &T) -> Self;
}

#[async_trait]
pub trait RpcWsProvider: Provider {
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response>;
}

const MAX_PRIORITY: u64 = 100;

pub enum Priority {
    Max,
    High,
    Normal,
    Low,
    Disabled,
    Custom(u64),
}

impl TryInto<PriorityValue> for Priority {
    type Error = RpcError;

    fn try_into(self) -> Result<PriorityValue, Self::Error> {
        match self {
            Self::Max => PriorityValue::new(MAX_PRIORITY),
            Self::High => PriorityValue::new(MAX_PRIORITY / 4 + MAX_PRIORITY / 2),
            Self::Normal => PriorityValue::new(MAX_PRIORITY / 2),
            Self::Low => PriorityValue::new(MAX_PRIORITY / 4),
            Self::Disabled => PriorityValue::new(0),
            Self::Custom(value) => PriorityValue::new(value),
        }
    }
}

#[derive(Debug)]
pub struct PriorityValue(u64);

impl PriorityValue {
    fn new(value: u64) -> RpcResult<Self> {
        if value > MAX_PRIORITY {
            return Err(anyhow::anyhow!(
                "Priority value cannot be greater than {}",
                MAX_PRIORITY
            ))
            .map_err(RpcError::from);
        }

        Ok(Self(value))
    }

    fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub struct Weight {
    value: std::sync::atomic::AtomicU64,
    priority: PriorityValue,
}

impl Weight {
    pub fn new(priority: Priority) -> RpcResult<Self> {
        let priority_val = TryInto::<PriorityValue>::try_into(priority)?.value();
        Ok(Self {
            value: std::sync::atomic::AtomicU64::new(priority_val),
            priority: PriorityValue::new(priority_val)?,
        })
    }

    pub fn value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn update_value(&self, value: u64) {
        self.value.store(
            // Calulate the new value based on the priority, with MAX_PRIORITY/2 being the "normal"
            // value Everything above MAX_PRIORITY/2 will be prioritized, everything
            // below will be deprioritized
            (value * self.priority.value()) / (MAX_PRIORITY / 2),
            std::sync::atomic::Ordering::SeqCst,
        );
    }
}

#[derive(Debug)]
pub struct SupportedChain {
    pub chain_id: String,
    pub weight: Weight,
}

pub trait Provider: Send + Sync + Debug + RateLimited {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool;

    fn supported_caip_chains(&self) -> Vec<String>;

    fn provider_kind(&self) -> ProviderKind;
}

#[async_trait]
pub trait RateLimited {
    async fn is_rate_limited(&self, data: &mut Response) -> bool;
}

#[async_trait]
pub trait HistoryProvider: Send + Sync + Debug {
    async fn get_transactions(
        &self,
        address: String,
        body: hyper::body::Bytes,
        params: HistoryQueryParams,
    ) -> RpcResult<HistoryResponseBody>;
}
