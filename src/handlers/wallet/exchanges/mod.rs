use {
    crate::{state::AppState, utils::crypto::Caip19Asset},
    axum::extract::State,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    strum::IntoEnumIterator,
    strum_macros::{AsRefStr, EnumIter},
    thiserror::Error,
};

pub mod binance;
pub mod coinbase;

use binance::BinanceExchange;
use coinbase::CoinbaseExchange;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub coinbase_project_id: Option<String>,
    pub coinbase_key_name: Option<String>,
    pub coinbase_key_secret: Option<String>,
    pub binance_client_id: Option<String>,
    pub binance_token: Option<String>,
    pub binance_key: Option<String>,
    pub binance_host: Option<String>,
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
}

pub struct GetBuyStatusParams {
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
}

#[derive(Debug, Clone, Copy, EnumIter, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ExchangeType {
    Binance,
    Coinbase,
}

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Get pay url error: {0}")]
    GetPayUrlError(String),

    #[error("Internal error")]
    InternalError(String),
}

impl ExchangeType {
    pub fn provider(&self) -> Box<dyn ExchangeProvider> {
        match self {
            ExchangeType::Binance => Box::new(BinanceExchange),
            ExchangeType::Coinbase => Box::new(CoinbaseExchange),
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
        }
    }

    pub fn is_asset_supported(&self, asset: &Caip19Asset) -> bool {
        self.provider().is_asset_supported(asset)
    }
}

pub fn get_supported_exchanges(asset: Option<String>) -> Result<Vec<Exchange>, ExchangeError> {
    match asset {
        Some(asset_str) => {
            let asset = Caip19Asset::parse(&asset_str)
                .map_err(|e| ExchangeError::ValidationError(e.to_string()))?;
            Ok(ExchangeType::iter()
                .filter(|e| e.is_asset_supported(&asset))
                .map(|e| e.to_exchange())
                .collect())
        }
        None => Ok(ExchangeType::iter().map(|e| e.to_exchange()).collect()),
    }
}

pub fn get_exchange_by_id(id: &str) -> Option<Exchange> {
    ExchangeType::from_id(id).map(|e| e.to_exchange())
}
