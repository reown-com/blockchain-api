use {
    crate::{
        state::AppState,
        utils::crypto::{self, Caip19Asset},
    },
    axum::extract::State,
    cerberus::project::{Feature, ProjectDataRequest},
    serde::{Deserialize, Serialize},
    std::{net::IpAddr, sync::Arc},
    strum::{EnumProperty, IntoEnumIterator},
    strum_macros::{AsRefStr, Display, EnumIter},
    thiserror::Error,
    tracing::debug,
};

pub mod binance;
pub mod coinbase;
pub mod get_exchange_buy_status;
pub mod get_exchange_url;
pub mod get_exchanges;
pub mod reconciler;
pub mod test_exchange;
pub mod transactions;

use binance::BinanceExchange;
use coinbase::CoinbaseExchange;
use test_exchange::TestExchange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, AsRefStr, EnumProperty)]
pub enum FeatureType {
    #[strum(
        serialize = "payments",
        to_string = "Payments",
        props(feature_id = "payments")
    )]
    Payments,
    #[strum(
        serialize = "fund_from_exchange",
        to_string = "Fund Wallet",
        props(feature_id = "fund_from_exchange")
    )]
    FundWallet,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub coinbase_project_id: Option<String>,
    #[deprecated(note = "Deprecated in favor of internal_api_coinbase_credentials")]
    pub coinbase_key_name: Option<String>,
    #[deprecated(note = "Deprecated in favor of internal_api_coinbase_credentials")]
    pub coinbase_key_secret: Option<String>,
    pub internal_api_coinbase_credentials: Option<String>,
    pub binance_client_id: Option<String>,
    pub binance_token: Option<String>,
    pub binance_key: Option<String>,
    pub binance_host: Option<String>,
    pub allowed_project_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
}

pub struct GetBuyUrlParams {
    pub project_id: String,
    pub asset: Caip19Asset,
    pub amount: f64,
    pub recipient: String,
    pub session_id: String,
    pub user_ip: IpAddr,
}

pub struct GetBuyStatusParams {
    pub project_id: String,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BuyTransactionStatus {
    Unknown,
    InProgress,
    Success,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBuyStatusResponse {
    pub status: BuyTransactionStatus,
    pub tx_hash: Option<String>,
}

pub trait ExchangeProvider {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn image_url(&self) -> Option<&'static str>;
    fn is_asset_supported(&self, asset: &Caip19Asset) -> bool;
    fn to_exchange(&self) -> Exchange {
        Exchange {
            id: self.id().to_string(),
            name: self.name().to_string(),
            image_url: self.image_url().map(|s| s.to_string()),
        }
    }
    fn is_enabled(&self, feature_type: &FeatureType, project_features: &[Feature]) -> bool;
}

#[derive(Debug, Clone, Copy, EnumIter, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ExchangeType {
    Binance,
    Coinbase,
    ReownTest,
}

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Get pay url error: {0}")]
    GetPayUrlError(String),

    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),

    #[error("Exchange is not enabled: {0}")]
    ExchangeNotEnabled(String),

    #[error("Project data error: {0}")]
    ProjectDataError(String),

    #[error("Exchange internal error: {0}")]
    InternalError(String),
}

impl ExchangeType {
    pub fn provider(&self) -> Box<dyn ExchangeProvider> {
        match self {
            ExchangeType::Binance => Box::new(BinanceExchange),
            ExchangeType::Coinbase => Box::new(CoinbaseExchange),
            ExchangeType::ReownTest => Box::new(TestExchange),
        }
    }

    pub fn to_exchange(&self) -> Exchange {
        self.provider().to_exchange()
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::iter().find(|e| e.provider().id() == id)
    }

    pub async fn get_buy_url(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyUrlParams,
    ) -> Result<String, ExchangeError> {
        match self {
            ExchangeType::Binance => BinanceExchange.get_buy_url(state, params).await,
            ExchangeType::Coinbase => CoinbaseExchange.get_buy_url(state, params).await,
            ExchangeType::ReownTest => TestExchange.get_buy_url(state, params),
        }
    }

    pub async fn get_buy_status(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyStatusParams,
    ) -> Result<GetBuyStatusResponse, ExchangeError> {
        match self {
            ExchangeType::Binance => BinanceExchange.get_buy_status(state, params).await,
            ExchangeType::Coinbase => CoinbaseExchange.get_buy_status(state, params).await,
            ExchangeType::ReownTest => TestExchange.get_buy_status(state, params).await,
        }
    }

    pub fn is_asset_supported(&self, asset: &Caip19Asset) -> bool {
        self.provider().is_asset_supported(asset)
    }

    pub fn is_transaction_storage_enabled(&self) -> bool {
        match self {
            ExchangeType::Binance => true,
            ExchangeType::Coinbase => true,
            ExchangeType::ReownTest => false,
        }
    }

    pub fn is_enabled(&self, feature_type: &FeatureType, project_features: &[Feature]) -> bool {
        self.provider().is_enabled(feature_type, project_features)
    }
}

pub fn get_supported_exchanges(
    asset: Option<String>,
    feature_type: &FeatureType,
    project_features: &[Feature],
) -> Result<Vec<Exchange>, ExchangeError> {
    match asset {
        Some(asset_str) => {
            let asset = Caip19Asset::parse(&asset_str)
                .map_err(|e| ExchangeError::ValidationError(e.to_string()))?;
            Ok(ExchangeType::iter()
                .filter(|e| {
                    e.is_asset_supported(&asset) && e.is_enabled(feature_type, project_features)
                })
                .map(|e| e.to_exchange())
                .collect())
        }
        None => Ok(ExchangeType::iter()
            .filter(|e| e.is_enabled(feature_type, project_features))
            .map(|e| e.to_exchange())
            .collect()),
    }
}

pub fn get_exchange_by_id(
    id: &str,
    feature_type: &FeatureType,
    project_features: &[Feature],
) -> Result<ExchangeType, ExchangeError> {
    let exchange_type = ExchangeType::from_id(id)
        .ok_or_else(|| ExchangeError::ValidationError(format!("Exchange {} not found", id)))?;

    if !exchange_type.is_enabled(feature_type, project_features) {
        return Err(ExchangeError::ExchangeNotEnabled(format!(
            "Exchange {} is not enabled",
            id
        )));
    }

    Ok(exchange_type)
}

async fn get_enabled_features(
    state: State<Arc<AppState>>,
    project_id: &str,
) -> Result<Vec<Feature>, ExchangeError> {
    let request = ProjectDataRequest::new(project_id)
        .include_features()
        .include_limits();
    let project_data = state
        .registry
        .project_data_request(request)
        .await
        .map_err(|e| ExchangeError::ProjectDataError(e.to_string()))?;
    debug!("project_data: {:?}", project_data);
    let features = project_data.features.unwrap_or_default();
    Ok(features)
}

pub fn get_feature_type(source: Option<&str>) -> FeatureType {
    match source {
        Some("fund-wallet") => FeatureType::FundWallet,
        _ => FeatureType::Payments,
    }
}

pub async fn is_feature_enabled_for_project_id(
    state: State<Arc<AppState>>,
    project_id: &str,
    project_features: &[Feature],
    feature_type: &FeatureType,
) -> Result<(), ExchangeError> {
    if let Some(testing_project_id) = state.config.server.testing_project_id.as_ref() {
        if crypto::constant_time_eq(testing_project_id, project_id) {
            return Ok(());
        }
    }

    if let Some(allowed_project_ids) = state.config.exchanges.allowed_project_ids.as_ref() {
        debug!("allowed_project_ids: {:?}", allowed_project_ids);
        if allowed_project_ids.iter().any(|id| id == project_id) {
            return Ok(());
        }
    }

    debug!("features: {:?}", project_features);

    let feature_id = feature_type
        .get_str("feature_id")
        .unwrap_or_else(|| feature_type.as_ref());

    if project_features
        .iter()
        .any(|f| f.id == feature_id && f.is_enabled)
    {
        return Ok(());
    }

    Err(ExchangeError::FeatureNotEnabled(format!(
        "{} feature is not enabled for this project",
        feature_type
    )))
}
