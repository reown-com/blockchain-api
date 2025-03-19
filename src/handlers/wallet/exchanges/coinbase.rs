use crate::handlers::wallet::exchanges::ExchangeProvider;

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

    fn get_buy_url(&self, asset: &str, amount: &str) -> String {
        format!("https://coinbase.com/buy?asset={asset}&amount={amount}")
    }
}
