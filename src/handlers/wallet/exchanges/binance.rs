use crate::handlers::wallet::exchanges::{ExchangeError, ExchangeProvider};
use crate::state::AppState;
use axum::extract::State;
use std::sync::Arc;

pub struct BinanceExchange;

impl ExchangeProvider for BinanceExchange {
    fn id(&self) -> &'static str {
        "binance"
    }

    fn name(&self) -> &'static str {
        "Binance"
    }

    fn image_url(&self) -> Option<&'static str> {
        Some("https://cryptologos.cc/logos/binance-coin-bnb-logo.png")
    }
}

impl BinanceExchange {
    pub async fn get_buy_url(
        _state: State<Arc<AppState>>,
        asset: &str,
        amount: &str,
    ) -> Result<String, ExchangeError> {
        // TODO: Communicate with the Binance API to get the buy URL
        Ok(format!(
            "https://binance.com/buy?asset={asset}&amount={amount}"
        ))
    }
}
