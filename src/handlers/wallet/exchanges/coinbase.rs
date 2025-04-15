use {
    crate::handlers::wallet::exchanges::{ExchangeError, ExchangeProvider, GetBuyUrlParams, GetBuyStatusParams, GetBuyStatusResponse, BuyTransactionStatus},
    crate::state::AppState,
    crate::utils::crypto::Caip19Asset,
    axum::extract::State,
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    std::sync::Arc,
    url::Url,
    base64::engine::general_purpose::STANDARD,
    base64::prelude::*,
    std::time::{SystemTime, UNIX_EPOCH},
    rand::RngCore,
    ed25519_dalek::{Signer, SigningKey},
    tracing::debug,
};

const COINBASE_ONE_CLICK_BUY_URL: &str = "https://pay.coinbase.com/buy/select-asset";
const DEFAULT_PAYMENT_METHOD: &str = "CRYPTO_ACCOUNT";
const COINBASE_API_HOST: &str = "api.developer.coinbase.com";

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

#[derive(Debug, Serialize, Deserialize)]
struct TransactionStatusResponse {
    next_page_key: Option<String>,
    total_count: String,
    transactions: Vec<OnrampTransaction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OnrampTransactionStatus {
    OnrampTransactionStatusInProgress,
    OnrampTransactionStatusSuccess,
    OnrampTransactionStatusFailed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OnrampPaymentMethod {
    Card,
    AchBankAccount,
    ApplePay,
    FiatWallet,
    CryptoWallet,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OnrampTransactionType {
    OnrampTransactionTypeBuyAndSend,
    OnrampTransactionTypeSend,
}

#[derive(Debug, Serialize, Deserialize)]
struct OnrampTransaction {
    status: OnrampTransactionStatus,
    purchase_currency: String,
    purchase_network: String,
    purchase_amount: String, 
    payment_total: String,   
    payment_subtotal: String, 
    coinbase_fee: String,    
    network_fee: String,     
    exchange_rate: String,  
    country: String,
    user_id: String,
    payment_method: OnrampPaymentMethod,
    tx_hash: Option<String>, 
    transaction_id: String,
    wallet_address: String,
    #[serde(rename = "type")]
    transaction_type: OnrampTransactionType,
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

    fn get_api_credentials(
        &self,
        state: &Arc<AppState>,
    ) -> Result<(String, String), ExchangeError> {
        let key_name = state.config.exchanges.coinbase_key_name.clone();
        let key_secret = state.config.exchanges.coinbase_key_secret.clone();

        if key_name.is_none() || key_secret.is_none() {
            return Err(ExchangeError::ConfigurationError(
                "Exchange is not available".to_string(),
            ));
        }

        Ok((key_name.unwrap(), key_secret.unwrap()))
    }

    async fn send_get_request(
        &self,
        state: &Arc<AppState>,
        path: &str,
    ) -> Result<reqwest::Response, ExchangeError> {
        let (pub_key, priv_key) = self.get_api_credentials(state)?;

        let jwt_key =
            generate_coinbase_jwt_key(&pub_key, &priv_key, "GET", COINBASE_API_HOST, path)?;

        let url = format!("https://{}{}", COINBASE_API_HOST, path);

        let res = state
            .http_client
            .get(url)
            .bearer_auth(jwt_key)
            .send()
            .await
            .map_err(|e| ExchangeError::InternalError(e.to_string()))?;

        debug!("send_get_request response: {:?}", res);
        Ok(res)
    }


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

    async fn get_transaction_status(
        &self,
        state: &Arc<AppState>,
        transaction_id: &str,
    ) -> Result<TransactionStatusResponse, ExchangeError> {
        let res = self.send_get_request(state, &format!("/onramp/v1/buy/user/{}/transactions", transaction_id)).await?;
        let body: TransactionStatusResponse = res.json().await.map_err(|e| {
            debug!("Error parsing transaction status response: {:?}", e);
            ExchangeError::InternalError(e.to_string())
        })?;
        debug!("get_transaction_status body: {:?}", body);
        Ok(body)
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
            .append_pair("partnerUserId", &params.session_id)
            .append_pair("defaultAsset", &crypto)
            .append_pair("defaultPaymentMethod", DEFAULT_PAYMENT_METHOD)
            .append_pair("presetCryptoAmount", &params.amount.to_string())
            .append_pair("defaultNetwork", &network)
            .append_pair("addresses", &addresses)
            .append_pair("assets", &assets);

        Ok(url.to_string())
    }

    pub async fn get_buy_status(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyStatusParams,
    ) -> Result<GetBuyStatusResponse, ExchangeError> {

        let response = self.get_transaction_status(&state, &params.session_id).await?;

        debug!("get_buy_status response: {:?}", response);
        
        match response.transactions.first() {
            Some(transaction) => {
                let status = match transaction.status {
                    OnrampTransactionStatus::OnrampTransactionStatusInProgress => BuyTransactionStatus::InProgress,
                    OnrampTransactionStatus::OnrampTransactionStatusSuccess => BuyTransactionStatus::Success,
                    OnrampTransactionStatus::OnrampTransactionStatusFailed => BuyTransactionStatus::Failed,
                };
                Ok(GetBuyStatusResponse { status, tx_hash: transaction.tx_hash.clone() })
            }
            None => Ok(GetBuyStatusResponse { status: BuyTransactionStatus::Unknown, tx_hash: None }),
        }

    
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

fn generate_coinbase_jwt_key(
    key_name: &str,
    key_secret: &str,
    request_method: &str,
    request_host: &str,
    request_path: &str,
) -> Result<String, ExchangeError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ExchangeError::InternalError("Failed to get current time".to_string()))?
        .as_secs() as usize;
    let uri = format!("{} {}{}", request_method, request_host, request_path);
    let claims = Claims {
        iss: "cdp".to_string(),
        nbf: now,
        exp: now + 120,
        sub: key_name.to_string(),
        uri,
    };
    let mut nonce_bytes = [0u8; 16];
    rand::thread_rng()
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|_| ExchangeError::InternalError("Failed to generate nonce".to_string()))?;
    let nonce = hex::encode(nonce_bytes);
    let header = serde_json::json!({
        "alg": "EdDSA",
        "kid": key_name,
        "nonce": nonce,
        "typ": "JWT"
    });

    let header_b64 = BASE64_URL_SAFE_NO_PAD.encode(&serde_json::to_vec(&header).unwrap());
    let claims_b64 = BASE64_URL_SAFE_NO_PAD.encode(&serde_json::to_vec(&claims).unwrap());
    let message = format!("{}.{}", header_b64, claims_b64);

    let secret_bytes = STANDARD
        .decode(key_secret.trim())
        .map_err(|_| ExchangeError::InternalError("Failed to decode key secret".to_string()))?;

    let secret_array: [u8; 32] = secret_bytes[..32]
        .try_into()
        .map_err(|_| ExchangeError::InternalError("Invalid key length".to_string()))?;

    let signing_key = SigningKey::from_bytes(&secret_array);
    let signature = signing_key.sign(message.as_bytes());
    let signature_b64 = BASE64_URL_SAFE_NO_PAD.encode(&signature.to_bytes());

    Ok(format!("{}.{}.{}", header_b64, claims_b64, signature_b64))
}