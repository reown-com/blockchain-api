use {
    self::{coinbase::CoinbaseProvider, zerion::ZerionProvider},
    crate::{
        env::ProviderConfig,
        error::{RpcError, RpcResult},
        handlers::{
            balance::{self, BalanceQueryParams, BalanceResponseBody},
            convert::{
                allowance::{AllowanceQueryParams, AllowanceResponseBody},
                approve::{ConvertApproveQueryParams, ConvertApproveResponseBody},
                gas_price::{GasPriceQueryParams, GasPriceQueryResponseBody},
                quotes::{ConvertQuoteQueryParams, ConvertQuoteResponseBody},
                tokens::{TokensListQueryParams, TokensListResponseBody},
                transaction::{ConvertTransactionQueryParams, ConvertTransactionResponseBody},
            },
            fungible_price::{PriceCurrencies, PriceResponseBody},
            history::{HistoryQueryParams, HistoryResponseBody},
            onramp::{
                options::{OnRampBuyOptionsParams, OnRampBuyOptionsResponse},
                quotes::{OnRampBuyQuotesParams, OnRampBuyQuotesResponse},
            },
            portfolio::{PortfolioQueryParams, PortfolioResponseBody},
            RpcQueryParams,
        },
    },
    async_trait::async_trait,
    axum::response::Response,
    axum_tungstenite::WebSocketUpgrade,
    hyper::http::HeaderValue,
    rand::{distributions::WeightedIndex, prelude::Distribution, rngs::OsRng},
    serde::{Deserialize, Serialize},
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
        hash::Hash,
        sync::Arc,
    },
    tracing::{info, log::warn},
    wc::metrics::TaskMetrics,
};

mod aurora;
mod base;
mod binance;
mod coinbase;
mod getblock;
mod infura;
mod mantle;
mod near;
mod one_inch;
mod pokt;
mod publicnode;
mod quicknode;
mod weights;
pub mod zerion;
mod zksync;
mod zora;

pub use {
    aurora::AuroraProvider,
    base::BaseProvider,
    binance::BinanceProvider,
    getblock::GetBlockProvider,
    infura::{InfuraProvider, InfuraWsProvider},
    mantle::MantleProvider,
    near::NearProvider,
    one_inch::OneInchProvider,
    pokt::PoktProvider,
    publicnode::PublicnodeProvider,
    quicknode::QuicknodeProvider,
    zksync::ZKSyncProvider,
    zora::{ZoraProvider, ZoraWsProvider},
};

static WS_PROXY_TASK_METRICS: TaskMetrics = TaskMetrics::new("ws_proxy_task");

pub type WeightResolver = HashMap<String, HashMap<ProviderKind, Weight>>;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ProvidersConfig {
    pub prometheus_query_url: Option<String>,
    pub prometheus_workspace_header: Option<String>,

    pub infura_project_id: String,
    pub pokt_project_id: String,
    pub quicknode_api_token: String,

    pub zerion_api_key: Option<String>,
    pub coinbase_api_key: Option<String>,
    pub coinbase_app_id: Option<String>,
    pub one_inch_api_key: Option<String>,
    /// GetBlock provider access tokens in JSON format
    pub getblock_access_tokens: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupportedChains {
    pub http: HashSet<String>,
    pub ws: HashSet<String>,
}

pub struct ProviderRepository {
    pub supported_chains: SupportedChains,

    providers: HashMap<ProviderKind, Arc<dyn RpcProvider>>,
    ws_providers: HashMap<ProviderKind, Arc<dyn RpcWsProvider>>,

    weight_resolver: WeightResolver,
    ws_weight_resolver: WeightResolver,

    prometheus_client: prometheus_http_query::Client,
    prometheus_workspace_header: String,

    pub history_provider: Arc<dyn HistoryProvider>,
    pub portfolio_provider: Arc<dyn PortfolioProvider>,
    pub coinbase_pay_provider: Arc<dyn HistoryProvider>,
    pub onramp_provider: Arc<dyn OnRampProvider>,
    pub balance_provider: Arc<dyn BalanceProvider>,
    pub conversion_provider: Arc<dyn ConversionProvider>,
    pub fungible_price_provider: Arc<dyn FungiblePriceProvider>,
}

impl ProviderRepository {
    #[allow(clippy::new_without_default)]
    pub fn new(config: &ProvidersConfig) -> Self {
        let prometheus_client = {
            let prometheus_query_url = config
                .prometheus_query_url
                .clone()
                .unwrap_or("http://localhost:8080/".into());

            prometheus_http_query::Client::try_from(prometheus_query_url)
                .expect("Failed to connect to prometheus")
        };

        let prometheus_workspace_header = config
            .prometheus_workspace_header
            .clone()
            .unwrap_or("localhost:9090".into());

        // Don't crash the application if the ZERION_API_KEY is not set
        // TODO: find a better way to handle this
        let zerion_api_key = config
            .zerion_api_key
            .clone()
            .unwrap_or("ZERION_KEY_UNDEFINED".into());

        // Don't crash the application if the COINBASE_API_KEY_UNDEFINED is not set
        // TODO: find a better way to handle this
        let coinbase_api_key = config
            .coinbase_api_key
            .clone()
            .unwrap_or("COINBASE_API_KEY_UNDEFINED".into());

        // Don't crash the application if the COINBASE_APP_ID_UNDEFINED is not set
        // TODO: find a better way to handle this
        let coinbase_app_id = config
            .coinbase_app_id
            .clone()
            .unwrap_or("COINBASE_APP_ID_UNDEFINED".into());

        // Don't crash the application if the ONE_INCH_API_KEY is not set
        // TODO: find a better way to handle this
        let one_inch_api_key = config
            .one_inch_api_key
            .clone()
            .unwrap_or("ONE_INCH_API_KEY".into());

        let zerion_provider = Arc::new(ZerionProvider::new(zerion_api_key));
        let history_provider = zerion_provider.clone();
        let portfolio_provider = zerion_provider.clone();
        let balance_provider = zerion_provider.clone();
        let fungible_price_provider = zerion_provider;
        let conversion_provider = Arc::new(OneInchProvider::new(one_inch_api_key));

        let coinbase_pay_provider = Arc::new(CoinbaseProvider::new(
            coinbase_api_key,
            coinbase_app_id,
            "https://pay.coinbase.com/api/v1".into(),
        ));

        Self {
            supported_chains: SupportedChains {
                http: HashSet::new(),
                ws: HashSet::new(),
            },
            providers: HashMap::new(),
            ws_providers: HashMap::new(),
            weight_resolver: HashMap::new(),
            ws_weight_resolver: HashMap::new(),
            prometheus_client,
            prometheus_workspace_header,
            history_provider,
            portfolio_provider,
            coinbase_pay_provider: coinbase_pay_provider.clone(),
            onramp_provider: coinbase_pay_provider,
            balance_provider,
            conversion_provider,
            fungible_price_provider,
        }
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn get_provider_for_chain_id(
        &self,
        chain_id: &str,
        max_providers: usize,
    ) -> Result<Vec<Arc<dyn RpcProvider>>, RpcError> {
        let Some(providers) = self.weight_resolver.get(chain_id) else {
            return Err(RpcError::UnsupportedChain(chain_id.to_string()));
        };

        if providers.is_empty() {
            return Err(RpcError::UnsupportedChain(chain_id.to_string()));
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| weight.value()).collect();
        let non_zero_weight_providers = weights.iter().filter(|&x| *x > 0).count();
        let keys = providers.keys().cloned().collect::<Vec<_>>();

        match WeightedIndex::new(weights) {
            Ok(mut dist) => {
                let providers_to_iterate = std::cmp::min(max_providers, non_zero_weight_providers);
                let providers_result = (0..providers_to_iterate)
                    .map(|i| {
                        let dist_key = dist.sample(&mut OsRng);
                        let provider = keys.get(dist_key).ok_or_else(|| {
                            RpcError::WeightedProvidersIndex(format!(
                                "Failed to get random provider for chain_id: {}",
                                chain_id
                            ))
                        })?;

                        // Update the weight of the provider to 0 to remove it from the next
                        // sampling, as updating weights returns an error if
                        // all weights are zero
                        if i < providers_to_iterate - 1 {
                            if let Err(e) = dist.update_weights(&[(dist_key, &0)]) {
                                return Err(RpcError::WeightedProvidersIndex(format!(
                                    "Failed to update weight in sampling iteration: {}",
                                    e
                                )));
                            }
                        };

                        self.providers.get(provider).cloned().ok_or_else(|| {
                            RpcError::WeightedProvidersIndex(format!(
                                "Provider not found during the weighted index check: {}",
                                provider
                            ))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(providers_result)
            }
            Err(e) => {
                // Respond with temporarily unavailable when all weights are 0 for
                // a chain providers
                warn!("Failed to create weighted index: {}", e);
                Err(RpcError::ChainTemporarilyUnavailable(chain_id.to_string()))
            }
        }
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn get_ws_provider_for_chain_id(&self, chain_id: &str) -> Option<Arc<dyn RpcWsProvider>> {
        let providers = self.ws_weight_resolver.get(chain_id)?;
        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| weight.value()).collect();
        let keys = providers.keys().cloned().collect::<Vec<_>>();
        match WeightedIndex::new(weights) {
            Ok(dist) => {
                let random = dist.sample(&mut OsRng);
                let provider = keys
                    .get(random)
                    .expect("Failed to get random provider: out of index");

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
                self.supported_chains.ws.insert(chain_id.clone());
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
                self.supported_chains.http.insert(chain_id.clone());
                self.weight_resolver
                    .entry(chain_id)
                    .or_default()
                    .insert(provider_kind, weight);
            });
        info!("Added provider: {}", provider_kind);
    }

    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn get_provider_by_provider_id(&self, provider_id: &str) -> Option<Arc<dyn RpcProvider>> {
        let provider = ProviderKind::from_str(provider_id)?;

        self.providers.get(&provider).cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Aurora,
    Infura,
    Pokt,
    Binance,
    ZKSync,
    Publicnode,
    Base,
    Zora,
    Zerion,
    Coinbase,
    Quicknode,
    Near,
    Mantle,
    GetBlock,
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ProviderKind::Aurora => "Aurora",
            ProviderKind::Infura => "Infura",
            ProviderKind::Pokt => "Pokt",
            ProviderKind::Binance => "Binance",
            ProviderKind::ZKSync => "zkSync",
            ProviderKind::Publicnode => "Publicnode",
            ProviderKind::Base => "Base",
            ProviderKind::Zora => "Zora",
            ProviderKind::Zerion => "Zerion",
            ProviderKind::Coinbase => "Coinbase",
            ProviderKind::Quicknode => "Quicknode",
            ProviderKind::Near => "Near",
            ProviderKind::Mantle => "Mantle",
            ProviderKind::GetBlock => "GetBlock",
        })
    }
}

#[allow(clippy::should_implement_trait)]
impl ProviderKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Aurora" => Some(Self::Aurora),
            "Infura" => Some(Self::Infura),
            "Pokt" => Some(Self::Pokt),
            "Binance" => Some(Self::Binance),
            "zkSync" => Some(Self::ZKSync),
            "Publicnode" => Some(Self::Publicnode),
            "Base" => Some(Self::Base),
            "Zora" => Some(Self::Zora),
            "Zerion" => Some(Self::Zerion),
            "Coinbase" => Some(Self::Coinbase),
            "Quicknode" => Some(Self::Quicknode),
            "Near" => Some(Self::Near),
            "Mantle" => Some(Self::Mantle),
            "GetBlock" => Some(Self::GetBlock),
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

#[derive(Debug, Clone, Copy)]
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
        params: HistoryQueryParams,
        http_client: reqwest::Client,
    ) -> RpcResult<HistoryResponseBody>;
}

#[async_trait]
pub trait PortfolioProvider: Send + Sync + Debug {
    async fn get_portfolio(
        &self,
        address: String,
        body: hyper::body::Bytes,
        params: PortfolioQueryParams,
    ) -> RpcResult<PortfolioResponseBody>;
}

#[async_trait]
pub trait OnRampProvider: Send + Sync + Debug {
    async fn get_buy_options(
        &self,
        params: OnRampBuyOptionsParams,
        http_client: reqwest::Client,
    ) -> RpcResult<OnRampBuyOptionsResponse>;

    async fn get_buy_quotes(
        &self,
        params: OnRampBuyQuotesParams,
        http_client: reqwest::Client,
    ) -> RpcResult<OnRampBuyQuotesResponse>;
}

#[async_trait]
pub trait BalanceProvider: Send + Sync + Debug {
    async fn get_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        http_client: reqwest::Client,
    ) -> RpcResult<BalanceResponseBody>;
}

#[async_trait]
pub trait FungiblePriceProvider: Send + Sync + Debug {
    async fn get_price(
        &self,
        chain_id: &str,
        address: &str,
        currency: &PriceCurrencies,
        http_client: reqwest::Client,
    ) -> RpcResult<PriceResponseBody>;
}

#[async_trait]
pub trait ConversionProvider: Send + Sync + Debug {
    async fn get_tokens_list(
        &self,
        params: TokensListQueryParams,
    ) -> RpcResult<TokensListResponseBody>;

    async fn get_convert_quote(
        &self,
        params: ConvertQuoteQueryParams,
    ) -> RpcResult<ConvertQuoteResponseBody>;

    async fn build_approve_tx(
        &self,
        params: ConvertApproveQueryParams,
    ) -> RpcResult<ConvertApproveResponseBody>;

    async fn build_convert_tx(
        &self,
        params: ConvertTransactionQueryParams,
    ) -> RpcResult<ConvertTransactionResponseBody>;

    async fn get_gas_price(
        &self,
        params: GasPriceQueryParams,
    ) -> RpcResult<GasPriceQueryResponseBody>;

    async fn get_allowance(&self, params: AllowanceQueryParams)
        -> RpcResult<AllowanceResponseBody>;
}
