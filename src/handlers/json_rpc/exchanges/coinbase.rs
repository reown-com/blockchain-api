use {
    crate::handlers::json_rpc::exchanges::{
        BuyTransactionStatus, ExchangeError, ExchangeProvider, GetBuyStatusParams,
        GetBuyStatusResponse, GetBuyUrlParams,
    },
    crate::state::AppState,
    crate::utils::crypto::Caip19Asset,
    axum::extract::State,
    base64::engine::general_purpose::STANDARD,
    base64::prelude::*,
    ed25519_dalek::{Signer, SigningKey},
    once_cell::sync::Lazy,
    rand::RngCore,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    std::sync::Arc,
    std::time::{SystemTime, UNIX_EPOCH},
    tracing::debug,
    url::Url,
};

const COINBASE_ONE_CLICK_BUY_URL: &str = "https://pay.coinbase.com/buy/select-asset";
const DEFAULT_PAYMENT_METHOD: &str = "CRYPTO_ACCOUNT";
const COINBASE_API_HOST: &str = "api.developer.coinbase.com";

// CAIP-19 asset mappings to Coinbase assets
static CAIP19_TO_COINBASE_CRYPTO: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        (
            "eip155:8453/erc20:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
            "USDC",
        ), // USDC on Base
        (
            "eip155:10/erc20:0x94b008aA00579c1307B0EF2c499aD98a8ce58e58",
            "USDC",
        ), // USDC on Optimism
        (
            "eip155:42161/erc20:0xAf88d065E77C8Ccc2239327C5EDb3A432268e5831",
            "USDC",
        ), // USDC on Arbitrum
        (
            "eip155:137/erc20:0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
            "USDC",
        ), // USDC on Polygon
        (
            "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "USDC",
        ), // USDC on Ethereum
        ("eip155:1/slip44:60", "ETH"), // Native ETH
        (
            "eip155:1/erc20:0xdAC17F958D2ee523a2206206994597C13D831ec7",
            "USDT",
        ), // USDT on Ethereum
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp/token:EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "USDC",
        ), // USDC on Solana
    ])
});

static CHAIN_ID_TO_COINBASE_NETWORK: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("eip155:8453", "base"),                               // Base
        ("eip155:10", "optimism"),                             // Optimism
        ("eip155:42161", "arbitrum"),                          // Arbitrum
        ("eip155:137", "polygon"),                             // Polygon
        ("eip155:1", "ethereum"),                              // Ethereum
        ("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", "solana"), // Solana
    ])
});

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrencyAmount {
    currency: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionStatusResponse {
    next_page_key: Option<String>,
    total_count: String,
    transactions: Vec<OnrampTransaction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum CoinbaseTransactionStatus {
    #[serde(rename = "ONRAMP_TRANSACTION_STATUS_IN_PROGRESS")]
    InProgress,
    #[serde(rename = "ONRAMP_TRANSACTION_STATUS_SUCCESS")]
    Success,
    #[serde(rename = "ONRAMP_TRANSACTION_STATUS_FAILED")]
    Failed,
    #[serde(rename = "ONRAMP_TRANSACTION_STATUS_CREATED")]
    Created,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OnrampPaymentMethod {
    Card,
    AchBankAccount,
    ApplePay,
    FiatWallet,
    CryptoWallet,
    Unspecified,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum OnrampTransactionType {
    OnrampTransactionTypeBuyAndSend,
    OnrampTransactionTypeSend,
}

#[derive(Debug, Serialize, Deserialize)]
struct OnrampTransaction {
    status: CoinbaseTransactionStatus,
    purchase_currency: Option<String>,
    purchase_network: Option<String>,
    purchase_amount: Option<CurrencyAmount>,
    payment_total: Option<CurrencyAmount>,
    payment_subtotal: Option<CurrencyAmount>,
    coinbase_fee: Option<CurrencyAmount>,
    network_fee: Option<CurrencyAmount>,
    exchange_rate: Option<CurrencyAmount>,
    country: Option<String>,
    user_id: Option<String>,
    payment_method: Option<OnrampPaymentMethod>,
    tx_hash: Option<String>,
    transaction_id: Option<String>,
    wallet_address: Option<String>,
    #[serde(rename = "type")]
    transaction_type: OnrampTransactionType,
    created_at: Option<String>,
    completed_at: Option<String>,
    partner_user_ref: Option<String>,
    user_type: Option<String>,
    contract_address: Option<String>,
    failure_reason: Option<String>,
    end_partner_name: Option<String>,
    payment_total_usd: Option<CurrencyAmount>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionTokenAddresses {
    address: String,
    blockchains: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionTokenRequest {
    addresses: Vec<SessionTokenAddresses>,
    assets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionTokenResponse {
    token: String,
    channel_id: Option<String>,
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

        match (key_name, key_secret) {
            (Some(key_name), Some(key_secret)) => Ok((key_name, key_secret)),
            _ => Err(ExchangeError::ConfigurationError(
                "Exchange is not available".to_string(),
            )),
        }
    }

    async fn send_get_request(
        &self,
        state: &Arc<AppState>,
        path: &str,
    ) -> Result<reqwest::Response, ExchangeError> {
        let (pub_key, priv_key) = self.get_api_credentials(state)?;

        let jwt_key =
            generate_coinbase_jwt_key(&pub_key, &priv_key, "GET", COINBASE_API_HOST, path)?;

        let url = format!("https://{COINBASE_API_HOST}{path}");

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

    async fn send_post_request<T: serde::Serialize>(
        &self,
        state: &Arc<AppState>,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, ExchangeError> {
        let (pub_key, priv_key) = self.get_api_credentials(state)?;

        let jwt_key =
            generate_coinbase_jwt_key(&pub_key, &priv_key, "POST", COINBASE_API_HOST, path)?;

        let url = format!("https://{COINBASE_API_HOST}{path}");

        let res = state
            .http_client
            .post(url)
            .bearer_auth(jwt_key)
            .json(body)
            .send()
            .await
            .map_err(|e| ExchangeError::InternalError(e.to_string()))?;

        debug!("send_post_request response: {:?}", res);
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
                ExchangeError::ValidationError(format!("Unsupported asset: {full_caip19}"))
            })?
            .to_string();

        let network = CHAIN_ID_TO_COINBASE_NETWORK
            .get(chain_id.as_str())
            .ok_or_else(|| {
                ExchangeError::ValidationError(format!("Unsupported chain ID: {chain_id}"))
            })?
            .to_string();

        Ok((crypto, network))
    }

    async fn get_transaction_status(
        &self,
        state: &Arc<AppState>,
        transaction_id: &str,
    ) -> Result<TransactionStatusResponse, ExchangeError> {
        let res = self
            .send_get_request(
                state,
                &format!("/onramp/v1/buy/user/{transaction_id}/transactions"),
            )
            .await?;

        let body: TransactionStatusResponse = res.json().await.map_err(|e| {
            debug!("Error parsing transaction status response: {:?}", e);
            ExchangeError::InternalError(e.to_string())
        })?;
        debug!("get_transaction_status body: {:?}", body);
        Ok(body)
    }

    async fn generate_session_token(
        &self,
        state: &Arc<AppState>,
        asset: &Caip19Asset,
        recipient: &str,
    ) -> Result<String, ExchangeError> {
        let (crypto, network) = self.map_asset_to_coinbase_format(asset)?;

        let address_info = SessionTokenAddresses {
            address: recipient.to_string(),
            blockchains: vec![network],
        };

        let request = SessionTokenRequest {
            addresses: vec![address_info],
            assets: vec![crypto],
        };

        let res = self
            .send_post_request(state, "/onramp/v1/token", &request)
            .await?;

        if !res.status().is_success() {
            return Err(ExchangeError::InternalError(format!(
                "Failed to generate session token: {}",
                res.status()
            )));
        }

        let session_response: SessionTokenResponse = res.json().await.map_err(|e| {
            debug!("Error parsing session token response: {:?}", e);
            ExchangeError::InternalError(e.to_string())
        })?;

        debug!("Generated session token successfully");
        Ok(session_response.token)
    }

    pub async fn get_buy_url(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyUrlParams,
    ) -> Result<String, ExchangeError> {
        let (crypto, network) = self.map_asset_to_coinbase_format(&params.asset)?;

        let session_token = self
            .generate_session_token(&state, &params.asset, &params.recipient)
            .await?;

        let mut url = Url::parse(COINBASE_ONE_CLICK_BUY_URL)
            .map_err(|e| ExchangeError::InternalError(format!("Failed to parse URL: {e}")))?;

        url.query_pairs_mut()
            .append_pair("sessionToken", &session_token)
            .append_pair("partnerUserId", &params.session_id)
            .append_pair("defaultAsset", &crypto)
            .append_pair("defaultPaymentMethod", DEFAULT_PAYMENT_METHOD)
            .append_pair("presetCryptoAmount", &params.amount.to_string())
            .append_pair("defaultNetwork", &network)
            .append_pair("clientIp", &params.user_ip.to_string());

        Ok(url.to_string())
    }

    pub async fn get_buy_status(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyStatusParams,
    ) -> Result<GetBuyStatusResponse, ExchangeError> {
        let response = self
            .get_transaction_status(&state, &params.session_id)
            .await?;

        debug!("get_buy_status response: {:?}", response);

        match response.transactions.first() {
            Some(transaction) => {
                let tx_hash = transaction.tx_hash.clone();

                let status = match &transaction.status {
                    CoinbaseTransactionStatus::Success => {
                        if tx_hash.as_ref().is_none_or(String::is_empty) {
                            // It's possible that the transaction is successful
                            // but the tx_hash is not available yet.
                            BuyTransactionStatus::InProgress
                        } else {
                            BuyTransactionStatus::Success
                        }
                    }
                    CoinbaseTransactionStatus::InProgress => BuyTransactionStatus::InProgress,
                    CoinbaseTransactionStatus::Failed => BuyTransactionStatus::Failed,
                    CoinbaseTransactionStatus::Created => BuyTransactionStatus::InProgress,
                };

                Ok(GetBuyStatusResponse { status, tx_hash })
            }
            None => Ok(GetBuyStatusResponse {
                status: BuyTransactionStatus::Unknown,
                tx_hash: None,
            }),
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
    let uri = format!("{request_method} {request_host}{request_path}");
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
    let header = serde_json::to_vec(&header)
        .map_err(|e| ExchangeError::InternalError(format!("Failed to serialize header: {e}")))?;
    let header_b64 = BASE64_URL_SAFE_NO_PAD.encode(&header);
    let claims = serde_json::to_vec(&claims)
        .map_err(|e| ExchangeError::InternalError(format!("Failed to serialize claims: {e}")))?;
    let claims_b64 = BASE64_URL_SAFE_NO_PAD.encode(&claims);
    let message = format!("{header_b64}.{claims_b64}");

    let secret_bytes = STANDARD
        .decode(key_secret.trim())
        .map_err(|_| ExchangeError::InternalError("Failed to decode key secret".to_string()))?;

    let secret_array: [u8; 32] = secret_bytes[..32]
        .try_into()
        .map_err(|_| ExchangeError::InternalError("Invalid key length".to_string()))?;

    let signing_key = SigningKey::from_bytes(&secret_array);
    let signature = signing_key.sign(message.as_bytes());
    let signature_b64 = BASE64_URL_SAFE_NO_PAD.encode(signature.to_bytes());

    Ok(format!("{header_b64}.{claims_b64}.{signature_b64}"))
}
