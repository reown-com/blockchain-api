use crate::state::AppState;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};
use thiserror::Error;
pub mod binance;
pub mod coinbase;

use binance::BinanceExchange;
use coinbase::CoinbaseExchange;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub coinbase_key_name: Option<String>,
    pub coinbase_key_secret: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
}

pub trait ExchangeProvider {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn image_url(&self) -> Option<&'static str>;

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
        asset: &str,
        amount: &str,
    ) -> Result<String, ExchangeError> {
        match self {
            ExchangeType::Binance => BinanceExchange::get_buy_url(state, asset, amount).await,
            ExchangeType::Coinbase => CoinbaseExchange::get_buy_url(state, asset, amount).await,
        }
    }
}

pub fn get_supported_exchanges() -> Vec<Exchange> {
    ExchangeType::iter().map(|e| e.to_exchange()).collect()
}

pub fn get_exchange_by_id(id: &str) -> Option<Exchange> {
    ExchangeType::from_id(id).map(|e| e.to_exchange())
}
