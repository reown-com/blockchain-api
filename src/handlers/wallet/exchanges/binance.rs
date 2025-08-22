use {
    crate::handlers::wallet::exchanges::{
        BuyTransactionStatus, ExchangeError, ExchangeProvider, GetBuyStatusParams,
        GetBuyStatusResponse, GetBuyUrlParams,
    },
    crate::state::AppState,
    crate::utils::crypto::Caip19Asset,
    axum::extract::State,
    base64::{engine::general_purpose::STANDARD, Engine},
    once_cell::sync::Lazy,
    openssl::{hash::MessageDigest, pkey::PKey, sign::Signer},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    std::sync::Arc,
    tracing::debug,
};

pub struct BinanceExchange;

const PRE_ORDER_PATH: &str = "/papi/v1/ramp/connect/buy/pre-order";
const QUERY_ORDER_DETAILS_PATH: &str = "/papi/v1/ramp/connect/order";
const FALLBACK_MERCHANT_NAME: &str = " ";

// CAIP-19 asset mappings to Binance assets
static CAIP19_TO_BINANCE_CRYPTO: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        (
            "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "USDC",
        ), // USDC on Ethereum
        (
            "eip155:137/erc20:0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
            "USDC",
        ), // USDC on Polygon
        (
            "eip155:8453/erc20:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
            "USDC",
        ), // USDC on Base
        (
            "eip155:42161/erc20:0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
            "USDC",
        ), // USDC on Arbitrum
        ("eip155:1/slip44:60", "ETH"), // Native ETH
        ("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp/slip44:501", "SOL"), // Native SOL
        (
            "eip155:1/erc20:0xdAC17F958D2ee523a2206206994597C13D831ec7",
            "USDT",
        ), // USDT on Ethereum
        (
            "eip155:42161/erc20:0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9",
            "USDT",
        ), // USDT on Arbitrum
        (
            "eip155:10/erc20:0x94b008aA00579c1307B0EF2c499aD98a8ce58e58",
            "USDT",
        ), // USDT on Optimism
        (
            "eip155:137/erc20:0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
            "USDT",
        ), // USDT on Polygon
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp/token:EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "USDC",
        ), // USDC on Solana
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp/token:Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            "USDT",
        ), // USDT on Solana
        (
            "eip155:30/erc20:0x2AcC95758f8b5F583470ba265EB685a8F45fC9D5",
            "RIF",
        ), // RIF on Rootstock
    ])
});

// CAIP-2 chain ID mappings to Binance networks
static CHAIN_ID_TO_BINANCE_NETWORK: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("eip155:1", "ETH"),                                // Ethereum
        ("eip155:137", "MATIC"),                            // Polygon
        ("eip155:8453", "BASE"),                            // Base
        ("eip155:42161", "ARBITRUM"),                       // Arbitrum
        ("eip155:10", "OPTIMISM"),                          // Optimism
        ("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", "SOL"), // Solana
        ("eip155:30", "RSK"),                               // Rootstock
    ])
});

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BinanceOrderStatus {
    Init,
    OnRampProcessing,
    OnRampCompleted,
    OffRampProcessing,
    WithdrawInit,
    WithdrawProcessing,
    Completed,
    OffRampFailed,
    WithdrawAbandoned,
    OnRampFailed,
    WithdrawFailed,
    FailedReserved,
    Unknown(usize),
}

impl From<usize> for BinanceOrderStatus {
    fn from(value: usize) -> Self {
        match value {
            0 => BinanceOrderStatus::Init,
            1 => BinanceOrderStatus::OnRampProcessing,
            2 => BinanceOrderStatus::OnRampCompleted,
            6 => BinanceOrderStatus::OffRampProcessing,
            10 => BinanceOrderStatus::WithdrawInit,
            11 => BinanceOrderStatus::WithdrawProcessing,
            20 => BinanceOrderStatus::Completed,
            95 => BinanceOrderStatus::OffRampFailed,
            96 => BinanceOrderStatus::WithdrawAbandoned,
            97 => BinanceOrderStatus::OnRampFailed,
            98 => BinanceOrderStatus::WithdrawFailed,
            99 => BinanceOrderStatus::FailedReserved,
            _ => BinanceOrderStatus::Unknown(value),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreOrderRequest {
    /// The unique order id from the partner side. Supports only letters and numbers.
    pub external_order_id: String,

    /// Crypto currency. If not specified, Binance Connect will automatically select/recommend a default crypto currency.
    /// Required for SEND_PRIMARY.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_currency: Option<String>,

    /// Specify whether the requested amount is in fiat:1 or crypto:2
    pub amount_type: i32,
    /// Requested amount. Fraction is 8
    pub requested_amount: String,

    /// The payment method code from payment method list API.
    pub pay_method_code: Option<String>,

    /// The payment method subcode from payment method list API.
    pub pay_method_sub_code: Option<String>,

    /// Crypto network
    pub network: String,

    /// Wallet address
    pub address: String,

    /// If blockchain required
    pub memo: Option<String>,

    /// The redirectUrl is for redirecting to your website if order is completed
    pub redirect_url: Option<String>,

    /// The redirectUrl is for redirecting to your website if order is failed
    pub fail_redirect_url: Option<String>,

    /// The redirectDeepLink is for redirecting to your APP if order is completed
    pub redirect_deep_link: Option<String>,

    /// The failRedirectDeepLink is for redirecting to your APP if order is failed
    pub fail_redirect_deep_link: Option<String>,

    /// The original client IP
    pub client_ip: Option<String>,

    /// The original client type: web/mobile
    pub client_type: Option<String>,

    /// Customization settings for the current order
    pub customization: Option<Customization>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Customization {
    send_primary: Option<bool>,
    merchant_display_name: Option<String>,
    net_receive: Option<bool>,
    lock_order_attributes: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
enum LockOrderAttributeType {
    All = 1,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreOrderResponseData {
    link: String,
    link_expire_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaymentMethodListRequest {
    pub fiat_currency: String,
    pub crypto_currency: String,
    pub total_amount: String,
    pub amount_type: usize,
}

#[derive(Debug, Serialize, Deserialize)]
enum AmountType {
    Fiat = 1,
    Crypto = 2,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentMethodListResponseData {
    payment_methods: Vec<PaymentMethod>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentMethod {
    pay_method_code: Option<String>,
    pay_method_sub_code: Option<String>,
    payment_method: Option<String>,
    fiat_min_limit: Option<String>,
    fiat_max_limit: Option<String>,
    crypto_min_limit: Option<String>,
    crypto_max_limit: Option<String>,
    p2p: Option<bool>,
    withdraw_restriction: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryOrderDetailsRequest {
    external_order_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryOrderDetailsResponse {
    status: usize,
    withdraw_tx_hash: Option<String>,
}

/// Base response structure for Binance API responses
#[derive(Debug, Deserialize, Serialize)]
struct BinanceResponse<T> {
    success: Option<bool>,
    code: String,
    message: Option<String>,
    data: Option<T>,
    status: Option<String>,
}

impl ExchangeProvider for BinanceExchange {
    fn id(&self) -> &'static str {
        "binance"
    }

    fn name(&self) -> &'static str {
        "Binance"
    }

    fn image_url(&self) -> Option<&'static str> {
        Some("https://pay-assets.reown.com/binance_128_128.webp")
    }

    fn is_asset_supported(&self, asset: &Caip19Asset) -> bool {
        self.map_asset_to_binance_format(asset).is_ok()
    }
}

impl BinanceExchange {
    fn get_api_credentials(
        &self,
        state: &Arc<AppState>,
    ) -> Result<(String, String, String, String), ExchangeError> {
        let client_id = state.config.exchanges.binance_client_id.clone();
        let key = state.config.exchanges.binance_key.clone();
        let token = state.config.exchanges.binance_token.clone();
        let host = state.config.exchanges.binance_host.clone();

        match (client_id, key, token, host) {
            (Some(client_id), Some(key), Some(token), Some(host)) => {
                Ok((client_id, key, token, host))
            }
            _ => Err(ExchangeError::ConfigurationError(
                "Exchange is not available".to_string(),
            )),
        }
    }

    fn generate_signature(
        &self,
        body: &str,
        timestamp: u64,
        private_key: &str,
    ) -> Result<String, ExchangeError> {
        let key_bytes = STANDARD.decode(private_key).map_err(|e| {
            ExchangeError::GetPayUrlError(format!("Failed to decode private key: {e}"))
        })?;

        let pkey = PKey::private_key_from_pkcs8(&key_bytes).map_err(|e| {
            ExchangeError::GetPayUrlError(format!("Failed to parse private key: {e}"))
        })?;

        let data_to_sign = if body.is_empty() || body == "{}" {
            timestamp.to_string()
        } else {
            format!("{body}{timestamp}")
        };
        debug!("Data to sign: {}", data_to_sign);

        let mut signer = Signer::new(MessageDigest::sha256(), &pkey)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to create signer: {e}")))?;

        signer
            .update(data_to_sign.as_bytes())
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to update signer: {e}")))?;
        let signature = signer
            .sign_to_vec()
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to sign data: {e}")))?;

        Ok(STANDARD.encode(&signature))
    }

    async fn send_post_request<T, R>(
        &self,
        state: &Arc<AppState>,
        path: &str,
        payload: &T,
    ) -> Result<R, ExchangeError>
    where
        T: Serialize,
        R: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let (client_id, private_key, token, host) = self.get_api_credentials(state)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| ExchangeError::GetPayUrlError("Failed to get current time".to_string()))?
            .as_millis() as u64;

        let body = serde_json::to_string(payload).map_err(|e| {
            ExchangeError::GetPayUrlError(format!("Failed to serialize request body: {e}"))
        })?;

        let signature = self.generate_signature(&body, timestamp, &private_key)?;

        let url = format!("{host}{path}");

        let response = state
            .http_client
            .post(url)
            .json(payload)
            .header("Content-Type", "application/json")
            .header("X-Tesla-ClientId", &client_id)
            .header("X-Tesla-Timestamp", timestamp.to_string())
            .header("X-Tesla-Signature", signature)
            .header("X-Tesla-SignAccessToken", token)
            .send()
            .await
            .map_err(|e| ExchangeError::GetPayUrlError(e.to_string()))?;

        debug!("Binance response: {:?}", response);
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            let message =
                format!("Binance API request failed with status: {status}, body: {error_body}");
            debug!("Binance API request failed: {}", message);
            return Err(ExchangeError::InternalError(message));
        }

        let parsed_response: BinanceResponse<R> = response.json().await.map_err(|e| {
            debug!("Unable to parse Binance response: {}", e);
            ExchangeError::InternalError(format!("Failed to parse Binance response: {e}"))
        })?;
        debug!("Parsed response: {:?}", parsed_response);
        if let Some(success) = parsed_response.success {
            if !success {
                return Err(ExchangeError::InternalError(format!(
                    "Binance API request failed with code: {}, message: {}",
                    parsed_response.code,
                    parsed_response.message.unwrap_or_default()
                )));
            }
        }

        parsed_response.data.ok_or_else(|| {
            ExchangeError::InternalError("No data returned from Binance".to_string())
        })
    }

    pub fn map_asset_to_binance_format(
        &self,
        asset: &Caip19Asset,
    ) -> Result<(String, String), ExchangeError> {
        let full_caip19 = asset.to_string();
        let chain_id = asset.chain_id().to_string();

        let crypto = CAIP19_TO_BINANCE_CRYPTO
            .get(full_caip19.as_str())
            .map(|v| v.to_string())
            .ok_or_else(|| {
                ExchangeError::ValidationError(format!("Unsupported asset: {full_caip19}"))
            })?;

        let network = CHAIN_ID_TO_BINANCE_NETWORK
            .get(chain_id.as_str())
            .ok_or_else(|| {
                ExchangeError::ValidationError(format!("Unsupported chain ID: {chain_id}"))
            })?
            .to_string();

        Ok((crypto, network))
    }

    pub async fn get_buy_url(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyUrlParams,
    ) -> Result<String, ExchangeError> {
        let (crypto_currency, network) = self
            .map_asset_to_binance_format(&params.asset)
            .map_err(|e| ExchangeError::ValidationError(e.to_string()))?;

        let project = state
            .registry
            .project_data(&params.project_id)
            .await
            .map_err(|e| {
                debug!("Failed to get project data: {}", e);
                ExchangeError::InternalError(format!("Failed to get project data: {e}"))
            })?;
        let project_name = if project.data.name.is_empty() {
            debug!("Project name is empty, using fallback name");
            FALLBACK_MERCHANT_NAME.to_string()
        } else {
            project.data.name
        };

        let request = PreOrderRequest {
            external_order_id: params.session_id,
            crypto_currency: Some(crypto_currency),
            amount_type: AmountType::Crypto as i32,
            requested_amount: params.amount.to_string(),
            pay_method_code: None,
            pay_method_sub_code: None,
            network,
            address: params.recipient,
            memo: None,
            redirect_url: None,
            fail_redirect_url: None,
            redirect_deep_link: None,
            fail_redirect_deep_link: None,
            client_ip: None,
            client_type: None,
            customization: Some(Customization {
                send_primary: Some(true),
                merchant_display_name: Some(project_name),
                net_receive: Some(true),
                lock_order_attributes: Some(vec![2, 3]),
            }),
        };

        let url = self.create_pre_order(&state, request).await?;
        Ok(url)
    }

    pub async fn get_buy_status(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyStatusParams,
    ) -> Result<GetBuyStatusResponse, ExchangeError> {
        let request = QueryOrderDetailsRequest {
            external_order_id: params.session_id,
        };

        let response: QueryOrderDetailsResponse = self
            .send_post_request(&state, QUERY_ORDER_DETAILS_PATH, &request)
            .await?;

        debug!("get_buy_status response: {:?}", response);

        let binance_status: BinanceOrderStatus = response.status.into();

        let status = match binance_status {
            BinanceOrderStatus::OnRampCompleted | BinanceOrderStatus::Completed => {
                BuyTransactionStatus::Success
            }
            BinanceOrderStatus::Init
            | BinanceOrderStatus::OnRampProcessing
            | BinanceOrderStatus::OffRampProcessing
            | BinanceOrderStatus::WithdrawInit
            | BinanceOrderStatus::WithdrawProcessing => BuyTransactionStatus::InProgress,
            BinanceOrderStatus::OffRampFailed
            | BinanceOrderStatus::WithdrawAbandoned
            | BinanceOrderStatus::OnRampFailed
            | BinanceOrderStatus::WithdrawFailed
            | BinanceOrderStatus::FailedReserved => BuyTransactionStatus::Failed,
            BinanceOrderStatus::Unknown(_) => BuyTransactionStatus::Unknown,
        };

        Ok(GetBuyStatusResponse {
            status,
            tx_hash: response.withdraw_tx_hash,
        })
    }

    pub async fn create_pre_order(
        &self,
        state: &Arc<AppState>,
        request: PreOrderRequest,
    ) -> Result<String, ExchangeError> {
        let data: PreOrderResponseData = self
            .send_post_request(state, PRE_ORDER_PATH, &request)
            .await?;
        Ok(data.link)
    }
}
