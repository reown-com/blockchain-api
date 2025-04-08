use {
    crate::handlers::wallet::exchanges::{ExchangeError, ExchangeProvider, GetBuyUrlParams},
    crate::state::AppState,
    crate::utils::crypto::Caip19Asset,
    axum::extract::State,
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    std::sync::Arc,
    url::Url,
};

const COINBASE_ONE_CLICK_BUY_URL: &str = "https://pay.coinbase.com/buy/select-asset";
const DEFAULT_PAYMENT_METHOD: &str = "CRYPTO_ACCOUNT";

// CAIP-19 asset mappings to Coinbase assets
static CAIP19_TO_COINBASE_CRYPTO: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        (
            "eip155:8453/erc20:0x833589fcd6edb6e08f4c7c32d4f71b54bda02913",
            "USDC",
        ), // USDC on Base
    ])
});

static CHAIN_ID_TO_COINBASE_NETWORK: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("eip155:8453", "base"), // Base
    ])
});

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
        Some("https://pay-assets.reown.com/coinbase_128_128.webp")
    }

    fn is_asset_supported(&self, asset: &Caip19Asset) -> bool {
        CAIP19_TO_COINBASE_CRYPTO.contains_key(asset.to_string().as_str())
    }
}

impl CoinbaseExchange {
    fn map_asset_to_coinbase_format(
        &self,
        asset: &Caip19Asset,
    ) -> Result<(String, String), ExchangeError> {
        let full_caip19 = asset.to_string();
        let chain_id = asset.chain_id().to_string();

        let crypto = CAIP19_TO_COINBASE_CRYPTO
            .get(full_caip19.as_str())
            .ok_or_else(|| {
                ExchangeError::ValidationError(format!("Unsupported asset: {}", full_caip19))
            })?
            .to_string();

        let network = CHAIN_ID_TO_COINBASE_NETWORK
            .get(chain_id.as_str())
            .ok_or_else(|| {
                ExchangeError::ValidationError(format!("Unsupported chain ID: {}", chain_id))
            })?
            .to_string();

        Ok((crypto, network))
    }

    pub async fn get_buy_url(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyUrlParams,
    ) -> Result<String, ExchangeError> {
        let project_id = state
            .config
            .exchanges
            .coinbase_project_id
            .as_ref()
            .ok_or_else(|| {
                ExchangeError::ConfigurationError("Coinbase exchange is not configured".to_string())
            })?;

        let (crypto, network) = self.map_asset_to_coinbase_format(&params.asset)?;

        let addresses = serde_json::to_string(&HashMap::from([(
            params.recipient.clone(),
            vec![network.clone()],
        )]))
        .map_err(|e| {
            ExchangeError::InternalError(format!("Failed to serialize addresses: {}", e))
        })?;

        let assets = serde_json::to_string(&vec![crypto.clone()]).map_err(|e| {
            ExchangeError::InternalError(format!("Failed to serialize assets: {}", e))
        })?;

        let mut url = Url::parse(COINBASE_ONE_CLICK_BUY_URL)
            .map_err(|e| ExchangeError::InternalError(format!("Failed to parse URL: {}", e)))?;

        url.query_pairs_mut()
            .append_pair("appId", project_id)
            .append_pair("defaultAsset", &crypto)
            .append_pair("defaultPaymentMethod", DEFAULT_PAYMENT_METHOD)
            .append_pair("presetCryptoAmount", &params.amount.to_string())
            .append_pair("defaultNetwork", &network)
            .append_pair("addresses", &addresses)
            .append_pair("assets", &assets);

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
