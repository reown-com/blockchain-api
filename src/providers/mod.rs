use {
    crate::env::ProviderConfig,
    axum::response::Response,
    axum_tungstenite::WebSocketUpgrade,
    rand::{distributions::WeightedIndex, prelude::Distribution, rngs::OsRng},
    std::{fmt::Debug, hash::Hash, sync::Arc},
    tracing::{info, log::warn},
};

mod binance;
mod infura;
mod omnia;
mod pokt;
mod publicnode;
#[cfg(feature = "dynamic-weights")]
mod weights;
mod zksync;

use {
    crate::{error::RpcResult, handlers::RpcQueryParams},
    async_trait::async_trait,
    std::{collections::HashMap, fmt::Display},
};
pub use {
    binance::BinanceProvider,
    infura::{InfuraProvider, InfuraWsProvider},
    omnia::OmniatechProvider,
    pokt::PoktProvider,
    publicnode::PublicnodeProvider,
    zksync::ZKSyncProvider,
};

pub type WeightResolver = HashMap<String, HashMap<ProviderKind, Weight>>;

pub struct ProviderRepository {
    providers: HashMap<ProviderKind, Arc<dyn RpcProvider>>,
    ws_providers: HashMap<ProviderKind, Arc<dyn RpcWsProvider>>,

    weight_resolver: WeightResolver,
    ws_weight_resolver: WeightResolver,

    #[cfg(feature = "dynamic-weights")]
    prometheus_client: prometheus_http_query::Client,
}

impl ProviderRepository {
    pub fn new() -> Self {
        #[cfg(feature = "dynamic-weights")]
        let prometheus_client = {
            let prometheus_query_url =
                std::env::var("PROMETHEUS_QUERY_URL").unwrap_or("http://localhost:9090".into());
            prometheus_http_query::Client::try_from(prometheus_query_url)
                .expect("Failed to connect to prometheus")
        };

        Self {
            providers: HashMap::new(),
            ws_providers: HashMap::new(),
            weight_resolver: HashMap::new(),
            ws_weight_resolver: HashMap::new(),
            #[cfg(feature = "dynamic-weights")]
            prometheus_client,
        }
    }

    pub fn get_provider_for_chain_id(&self, chain_id: &str) -> Option<Arc<dyn RpcProvider>> {
        let Some(providers) = self.weight_resolver.get(chain_id) else {return None};

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
        let Some(providers) = self.ws_weight_resolver.get(chain_id) else {return None};

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
        let supported_ws_chains = provider_config.supported_chains();

        supported_ws_chains
            .into_iter()
            .for_each(|(chain_id, (_, weight))| {
                self.ws_weight_resolver
                    .entry(chain_id)
                    .or_insert_with(HashMap::new)
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
                    .or_insert_with(HashMap::new)
                    .insert(provider_kind, weight);
            });
        info!("Added provider: {}", provider_kind);
    }

    #[cfg(feature = "dynamic-weights")]
    pub async fn update_weights(&self, metrics: &crate::Metrics) {
        info!("Updating weights");

        match self
            .prometheus_client
            .query("round(increase(provider_status_code_counter[1h]))")
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
            _ => None,
        }
    }
}

#[async_trait]
pub trait RpcProvider: Provider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        xpath: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response>;
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

#[derive(Debug)]
pub struct Weight(pub std::sync::atomic::AtomicU32);

impl Weight {
    pub fn value(&self) -> u32 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
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

pub enum RateLimitedData<'a> {
    Response(&'a Response),
    Body(&'a hyper::body::Bytes),
}

pub trait RateLimited {
    fn is_rate_limited(&self, data: RateLimitedData) -> bool;
}
