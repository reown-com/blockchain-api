use crate::handlers::wallet::exchanges::{ExchangeError, ExchangeProvider, GetBuyUrlParams};
use crate::state::AppState;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;
use std::collections::HashMap;

const COINBASE_ONE_CLICK_BUY_URL: &str = "https://pay.coinbase.com/buy/select-asset";


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PaymentMethod {
    Unspecified,
    Card,
    AchBankAccount,
    ApplePay,
    FiatWallet,
    CryptoAccount,
    GuestCheckoutCard,
    PayPal,
    Rtp,
    GuestCheckoutApplePay,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateBuyQuoteRequest {
    country: String,
    payment_amount: String,
    payment_currency: String,
    payment_method: PaymentMethod,
    purchase_currency: String,
    purcase_network: String,
    #[serde(default)]
    subdivision: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrencyAmount {
    currency: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateBuyQuoteResponse {
    coinbase_fee: CurrencyAmount,
    network_fee: CurrencyAmount,
    payment_subtotal: CurrencyAmount,
    payment_total: CurrencyAmount,
    purchase_amount: CurrencyAmount,
    quote_id: String,
}

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
        state: State<Arc<AppState>>,
        params: GetBuyUrlParams,
    ) -> Result<String, ExchangeError> {

        let project_id = state.config.exchanges.coinbase_project_id.as_ref()
            .ok_or_else(|| ExchangeError::ConfigurationError("Coinbas exchange is not configured".to_string()))?;

        let mut url = Url::parse(COINBASE_ONE_CLICK_BUY_URL).map_err(|e| ExchangeError::InternalError(e.to_string()))?;
        
        let mut addresses = HashMap::new();
        addresses.insert(
            params.recipient,
            vec!["base".to_string()]
        );
        let addresses_json = serde_json::to_string(&addresses)
            .map_err(|e| ExchangeError::InternalError(format!("Failed to serialize addresses: {}", e)))?;
        
        let assets = vec!["USDC".to_string()];
        let assets_json = serde_json::to_string(&assets)
            .map_err(|e| ExchangeError::InternalError(format!("Failed to serialize assets: {}", e)))?;

        url.query_pairs_mut().append_pair("appId", &project_id);
        url.query_pairs_mut().append_pair("defaultAsset", &"USDC");
        url.query_pairs_mut().append_pair("defaultPaymentMethod", &"CRYPTO_ACCOUNT");
        url.query_pairs_mut().append_pair("presetCryptoAmount", &params.amount.to_string());
        url.query_pairs_mut().append_pair("defaultNetwork", &"base");
        url.query_pairs_mut().append_pair("addresses", &addresses_json);
        url.query_pairs_mut().append_pair("assets", &assets_json);
        
        Ok(url.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    nbf: usize,
    exp: usize,
    sub: String,
    uri: String,
}

