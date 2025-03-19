use serde::Serialize;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

pub mod binance;
pub mod coinbase;

use binance::BinanceExchange;
use coinbase::CoinbaseExchange;

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
    fn get_buy_url(&self, asset: &str, amount: &str) -> String;

    fn to_exchange(&self) -> Exchange {
        Exchange {
            id: self.id().to_string(),
            name: self.name().to_string(),
            image_url: self.image_url().map(|s| s.to_string()),
        }
    }
}

// Define static instances for each exchange to avoid repeated instantiation
const BINANCE: BinanceExchange = BinanceExchange;
const COINBASE: CoinbaseExchange = CoinbaseExchange;

#[derive(Debug, Clone, Copy, EnumIter, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ExchangeType {
    Binance,
    Coinbase,
}

impl ExchangeType {
    pub fn provider(&self) -> &'static dyn ExchangeProvider {
        match self {
            ExchangeType::Binance => &BINANCE,
            ExchangeType::Coinbase => &COINBASE,
        }
    }

    pub fn to_exchange(&self) -> Exchange {
        self.provider().to_exchange()
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::iter().find(|e| e.provider().id() == id)
    }

    pub fn get_buy_url(&self, asset: &str, amount: &str) -> String {
        self.provider().get_buy_url(asset, amount)
    }
}

pub fn get_supported_exchanges() -> Vec<Exchange> {
    ExchangeType::iter().map(|e| e.to_exchange()).collect()
}

pub fn get_exchange_by_id(id: &str) -> Option<Exchange> {
    ExchangeType::from_id(id).map(|e| e.to_exchange())
}

pub fn get_exchange_buy_url(id: &str, asset: &str, amount: &str) -> Option<String> {
    ExchangeType::from_id(id).map(|e| e.get_buy_url(asset, amount))
}
