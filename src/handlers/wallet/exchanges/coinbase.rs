use crate::handlers::wallet::exchanges::ExchangeProvider;
use crate::state::AppState;
use axum::extract::State;
use std::sync::Arc;

pub struct CoinbaseExchange;

impl ExchangeProvider for CoinbaseExchange {
    fn id(&self) -> &'static str {
        "coinbase"
    }

    fn name(&self) -> &'static str {
        "Coinbase"
    }

    fn image_url(&self) -> Option<&'static str> {
        Some("https://cdn.iconscout.com/icon/free/png-256/free-coinbase-logo-icon-download-in-svg-png-gif-file-formats--web-crypro-trading-platform-logos-pack-icons-7651204.png")
    }
}

impl CoinbaseExchange {
    pub async fn get_buy_url(
        _state: State<Arc<AppState>>,
        asset: &str,
        amount: &str,
    ) -> Option<String> {
        // TODO: Communicate with the Coinbase API to get the buy URL
        Some(format!(
            "https://coinbase.com/buy?asset={asset}&amount={amount}"
        ))
    }
}
