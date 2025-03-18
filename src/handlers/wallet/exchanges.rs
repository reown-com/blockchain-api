use serde::Serialize;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Copy, EnumIter, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum ExchangeType {
    Binance,
    Coinbase,
}

impl ExchangeType {
    pub fn to_exchange(&self) -> Exchange {
        match self {
            ExchangeType::Binance => Exchange {
                id: self.as_ref().to_string(),
                name: "Binance".to_string(),
                image_url: Some(
                    "https://cryptologos.cc/logos/binance-coin-bnb-logo.png".to_string(),
                ),
            },
            ExchangeType::Coinbase => Exchange {
                id: self.as_ref().to_string(),
                name: "Coinbase".to_string(),
                image_url: Some("https://cdn.iconscout.com/icon/free/png-256/free-coinbase-logo-icon-download-in-svg-png-gif-file-formats--web-crypro-trading-platform-logos-pack-icons-7651204.png".to_string()),
            },
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "binance" => Some(ExchangeType::Binance),
            "coinbase" => Some(ExchangeType::Coinbase),
            _ => None,
        }
    }
}

/// Returns a list of all supported exchanges
pub fn get_supported_exchanges() -> Vec<Exchange> {
    ExchangeType::iter().map(|e| e.to_exchange()).collect()
}

/// Returns a specific exchange by ID if it exists
pub fn get_exchange_by_id(id: &str) -> Option<Exchange> {
    ExchangeType::from_id(id).map(|e| e.to_exchange())
}
