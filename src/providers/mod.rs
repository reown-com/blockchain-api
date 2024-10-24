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
            fungible_price::PriceResponseBody,
            history::{HistoryQueryParams, HistoryResponseBody},
            onramp::{
                options::{OnRampBuyOptionsParams, OnRampBuyOptionsResponse},
                quotes::{OnRampBuyQuotesParams, OnRampBuyQuotesResponse},
            },
            portfolio::{PortfolioQueryParams, PortfolioResponseBody},
            RpcQueryParams, SupportedCurrencies,
        },
        utils::crypto::CaipNamespaces,
        Metrics,
    },
    alloy::{
        primitives::{Address, U256},
        rpc::json_rpc::Id,
    },
    async_trait::async_trait,
    axum::response::Response,
    axum_tungstenite::WebSocketUpgrade,
    hyper::http::HeaderValue,
    mock_alto::{MockAltoProvider, MockAltoUrls},
    rand::{distributions::WeightedIndex, prelude::Distribution, rngs::OsRng},
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::{
        collections::{HashMap, HashSet},
        fmt::{Debug, Display},
        hash::Hash,
        sync::Arc,
    },
    tracing::{debug, error, log::warn},
    wc::metrics::TaskMetrics,
};

mod aurora;
mod base;
mod berachain;
mod binance;
mod bungee;
mod coinbase;
mod getblock;
mod infura;
mod mantle;
pub mod mock_alto;
mod near;
mod one_inch;
mod pimlico;
mod pokt;
mod publicnode;
mod quicknode;
mod solscan;
mod unichain;
mod weights;
pub mod zerion;
mod zksync;
mod zora;

pub use {
    aurora::AuroraProvider,
    base::BaseProvider,
    berachain::BerachainProvider,
    binance::BinanceProvider,
    bungee::BungeeProvider,
    getblock::GetBlockProvider,
    infura::{InfuraProvider, InfuraWsProvider},
    mantle::MantleProvider,
    near::NearProvider,
    one_inch::OneInchProvider,
    pimlico::PimlicoProvider,
    pokt::PoktProvider,
    publicnode::PublicnodeProvider,
    quicknode::QuicknodeProvider,
    solscan::SolScanProvider,
    unichain::UnichainProvider,
    zksync::ZKSyncProvider,
    zora::{ZoraProvider, ZoraWsProvider},
};

static WS_PROXY_TASK_METRICS: TaskMetrics = TaskMetrics::new("ws_proxy_task");

pub type WeightResolver = HashMap<String, HashMap<ProviderKind, Weight>>;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ProvidersConfig {
    pub prometheus_query_url: Option<String>,
    pub prometheus_workspace_header: Option<String>,

    /// Redis address for provider's responses caching
    pub cache_redis_addr: Option<String>,

    pub infura_project_id: String,
    pub pokt_project_id: String,
    pub quicknode_api_tokens: String,

    pub zerion_api_key: Option<String>,
    pub coinbase_api_key: Option<String>,
    pub coinbase_app_id: Option<String>,
    pub one_inch_api_key: Option<String>,
    pub one_inch_referrer: Option<String>,
    /// GetBlock provider access tokens in JSON format
    pub getblock_access_tokens: Option<String>,
    /// Pimlico API token key
    pub pimlico_api_key: String,
    /// SolScan API v1 and v2 token keys
    pub solscan_api_v1_token: String,
    pub solscan_api_v2_token: String,
    /// Bungee API key
    pub bungee_api_key: String,

    pub override_bundler_urls: Option<MockAltoUrls>,
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

    pub history_providers: HashMap<CaipNamespaces, Arc<dyn HistoryProvider>>,
    pub portfolio_provider: Arc<dyn PortfolioProvider>,
    pub coinbase_pay_provider: Arc<dyn HistoryProvider>,
    pub onramp_provider: Arc<dyn OnRampProvider>,
    pub balance_providers: HashMap<CaipNamespaces, Arc<dyn BalanceProvider>>,
    pub conversion_provider: Arc<dyn ConversionProvider>,
    pub fungible_price_providers: HashMap<CaipNamespaces, Arc<dyn FungiblePriceProvider>>,
    pub bundler_ops_provider: Arc<dyn BundlerOpsProvider>,
    pub chain_orchestrator_provider: Arc<dyn ChainOrchestrationProvider>,
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

        // Redis pool for providers responses caching where needed
        let mut redis_pool = None;
        if let Some(redis_addr) = &config.cache_redis_addr {
            let redis_builder = deadpool_redis::Config::from_url(redis_addr)
                .builder()
                .map_err(|e| {
                    error!(
                        "Failed to create redis pool builder for provider's responses caching: {:?}",
                        e
                    );
                })
                .expect("Failed to create redis pool builder for provider's responses caching, builder is None");

            redis_pool = Some(Arc::new(
                redis_builder
                    .runtime(deadpool_redis::Runtime::Tokio1)
                    .build()
                    .expect("Failed to create redis pool"),
            ));
        };

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
        let one_inch_referrer = config.one_inch_referrer.clone();
        if one_inch_referrer.is_none() {
            warn!("ONE_INCH_REFERRER is not set");
        }

        let zerion_provider = Arc::new(ZerionProvider::new(zerion_api_key));
        let one_inch_provider = Arc::new(OneInchProvider::new(one_inch_api_key, one_inch_referrer));
        let portfolio_provider = zerion_provider.clone();
        let solscan_provider = Arc::new(SolScanProvider::new(
            config.solscan_api_v1_token.clone(),
            config.solscan_api_v2_token.clone(),
            redis_pool.clone(),
        ));

        let mut balance_providers: HashMap<CaipNamespaces, Arc<dyn BalanceProvider>> =
            HashMap::new();
        balance_providers.insert(CaipNamespaces::Eip155, zerion_provider.clone());
        balance_providers.insert(CaipNamespaces::Solana, solscan_provider.clone());

        let mut history_providers: HashMap<CaipNamespaces, Arc<dyn HistoryProvider>> =
            HashMap::new();
        history_providers.insert(CaipNamespaces::Eip155, zerion_provider.clone());
        history_providers.insert(CaipNamespaces::Solana, solscan_provider.clone());

        let coinbase_pay_provider = Arc::new(CoinbaseProvider::new(
            coinbase_api_key,
            coinbase_app_id,
            "https://pay.coinbase.com/api/v1".into(),
        ));

        let bundler_ops_provider: Arc<dyn BundlerOpsProvider> =
            if let Some(override_bundler_url) = config.override_bundler_urls.clone() {
                Arc::new(MockAltoProvider::new(override_bundler_url))
            } else {
                Arc::new(PimlicoProvider::new(config.pimlico_api_key.clone()))
            };

        let mut fungible_price_providers: HashMap<CaipNamespaces, Arc<dyn FungiblePriceProvider>> =
            HashMap::new();
        fungible_price_providers.insert(CaipNamespaces::Eip155, one_inch_provider.clone());
        fungible_price_providers.insert(CaipNamespaces::Solana, solscan_provider.clone());

        let chain_orchestrator_provider =
            Arc::new(BungeeProvider::new(config.bungee_api_key.clone()));

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
            history_providers,
            portfolio_provider,
            coinbase_pay_provider: coinbase_pay_provider.clone(),
            onramp_provider: coinbase_pay_provider,
            balance_providers,
            conversion_provider: one_inch_provider.clone(),
            fungible_price_providers,
            bundler_ops_provider,
            chain_orchestrator_provider,
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

        let weights: Vec<_> = providers
            .iter()
            .map(|(_, weight)| weight.value())
            .map(|w| w.min(1))
            .collect();
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
        debug!("Added provider: {}", provider_kind);
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub async fn update_weights(&self, metrics: &crate::Metrics) {
        debug!("Updating weights");

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
    Berachain,
    Bungee,
    ZKSync,
    Publicnode,
    Base,
    Zora,
    Zerion,
    Coinbase,
    OneInch,
    Quicknode,
    Near,
    Mantle,
    GetBlock,
    SolScan,
    Unichain,
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ProviderKind::Aurora => "Aurora",
                ProviderKind::Infura => "Infura",
                ProviderKind::Pokt => "Pokt",
                ProviderKind::Binance => "Binance",
                ProviderKind::Berachain => "Berachain",
                ProviderKind::Bungee => "Bungee",
                ProviderKind::ZKSync => "zkSync",
                ProviderKind::Publicnode => "Publicnode",
                ProviderKind::Base => "Base",
                ProviderKind::Zora => "Zora",
                ProviderKind::Zerion => "Zerion",
                ProviderKind::Coinbase => "Coinbase",
                ProviderKind::OneInch => "OneInch",
                ProviderKind::Quicknode => "Quicknode",
                ProviderKind::Near => "Near",
                ProviderKind::Mantle => "Mantle",
                ProviderKind::GetBlock => "GetBlock",
                ProviderKind::SolScan => "SolScan",
                ProviderKind::Unichain => "Unichain",
            }
        )
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
            "Berachain" => Some(Self::Berachain),
            "Bungee" => Some(Self::Bungee),
            "zkSync" => Some(Self::ZKSync),
            "Publicnode" => Some(Self::Publicnode),
            "Base" => Some(Self::Base),
            "Zora" => Some(Self::Zora),
            "Zerion" => Some(Self::Zerion),
            "Coinbase" => Some(Self::Coinbase),
            "OneInch" => Some(Self::OneInch),
            "Quicknode" => Some(Self::Quicknode),
            "Near" => Some(Self::Near),
            "Mantle" => Some(Self::Mantle),
            "GetBlock" => Some(Self::GetBlock),
            "SolScan" => Some(Self::SolScan),
            "Unichain" => Some(Self::Unichain),
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
pub trait HistoryProvider: Send + Sync {
    async fn get_transactions(
        &self,
        address: String,
        params: HistoryQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<HistoryResponseBody>;
}

#[async_trait]
pub trait PortfolioProvider: Send + Sync + Debug {
    async fn get_portfolio(
        &self,
        address: String,
        params: PortfolioQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<PortfolioResponseBody>;
}

#[async_trait]
pub trait OnRampProvider: Send + Sync + Debug {
    async fn get_buy_options(
        &self,
        params: OnRampBuyOptionsParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<OnRampBuyOptionsResponse>;

    async fn get_buy_quotes(
        &self,
        params: OnRampBuyQuotesParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<OnRampBuyQuotesResponse>;
}

#[async_trait]
pub trait BalanceProvider: Send + Sync {
    async fn get_balance(
        &self,
        address: String,
        params: BalanceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<BalanceResponseBody>;
}

#[async_trait]
pub trait FungiblePriceProvider: Send + Sync {
    async fn get_price(
        &self,
        chain_id: &str,
        address: &str,
        currency: &SupportedCurrencies,
        metrics: Arc<Metrics>,
    ) -> RpcResult<PriceResponseBody>;
}

#[async_trait]
pub trait ConversionProvider: Send + Sync {
    async fn get_tokens_list(
        &self,
        params: TokensListQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<TokensListResponseBody>;

    async fn get_convert_quote(
        &self,
        params: ConvertQuoteQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<ConvertQuoteResponseBody>;

    async fn build_approve_tx(
        &self,
        params: ConvertApproveQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<ConvertApproveResponseBody>;

    async fn build_convert_tx(
        &self,
        params: ConvertTransactionQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<ConvertTransactionResponseBody>;

    async fn get_gas_price(
        &self,
        params: GasPriceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<GasPriceQueryResponseBody>;

    async fn get_allowance(
        &self,
        params: AllowanceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<AllowanceResponseBody>;
}

/// List of supported bundler operations
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum SupportedBundlerOps {
    #[serde(rename = "eth_getUserOperationReceipt")]
    EthGetUserOperationReceipt,
    #[serde(rename = "eth_sendUserOperation")]
    EthSendUserOperation,
    #[serde(rename = "eth_estimateUserOperationGas")]
    EthEstimateUserOperationGas,
    /// Paymaster sponsor UserOp
    #[serde(rename = "pm_sponsorUserOperation")]
    PmSponsorUserOperation,
    #[serde(rename = "pimlico_getUserOperationGasPrice")]
    PimlicoGetUserOperationGasPrice,
}

/// Provider for the bundler operations
#[async_trait]
pub trait BundlerOpsProvider: Send + Sync + Debug {
    /// Send JSON-RPC request to the bundler
    async fn bundler_rpc_call(
        &self,
        chain_id: &str,
        id: Id,
        jsonrpc: Arc<str>,
        method: &SupportedBundlerOps,
        params: serde_json::Value,
    ) -> RpcResult<serde_json::Value>;

    /// Maps the operations enum variant to its provider-specific operation string.
    fn to_provider_op(&self, op: &SupportedBundlerOps) -> String;
}

/// Provider for the chain orchestrator operations
#[async_trait]
pub trait ChainOrchestrationProvider: Send + Sync + Debug {
    async fn get_bridging_quotes(
        &self,
        from_chain_id: String,
        from_token_address: Address,
        to_chain_id: String,
        to_token_address: Address,
        amount: U256,
        user_address: Address,
    ) -> Result<Vec<Value>, RpcError>;

    async fn build_bridging_tx(&self, route: Value) -> Result<bungee::BungeeBuildTx, RpcError>;
}
