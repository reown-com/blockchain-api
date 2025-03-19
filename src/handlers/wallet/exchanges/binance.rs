use crate::handlers::wallet::exchanges::ExchangeProvider;
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

    fn get_buy_url(&self, asset: &str, amount: &str) -> String {
        format!("https://binance.com/buy?asset={asset}&amount={amount}")
    }
}
