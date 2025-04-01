use {
    crate::handlers::wallet::exchanges::{ExchangeError, ExchangeProvider, GetBuyUrlParams},
    crate::state::AppState,
    crate::utils::crypto::Caip19Asset,
    axum::extract::State,
    std::sync::Arc,
    std::collections::HashMap,
    once_cell::sync::Lazy,
    serde::{Serialize, Deserialize},
    base64::{Engine, engine::general_purpose::STANDARD},
    tracing::debug,
    openssl::{pkey::PKey, sign::Signer, hash::MessageDigest},
    uuid::Uuid,
};

pub struct BinanceExchange;

const PRE_ORDER_PATH: &str = "/papi/v1/ramp/connect/buy/pre-order";
const PAYMENT_METHOD_LIST_PATH: &str = "/papi/v1/ramp/connect/buy/payment-method-list"; 
const DEFAULT_FIAT_CURRENCY: &str = "USD";
const DEFAULT_PAYMENT_METHOD_CODE: &str = "BUY_WALLET";
const DEFAULT_PAYMENT_METHOD_SUB_CODE: &str = "Wallet";

// CAIP-19 asset mappings to Binance assets
static CAIP19_TO_BINANCE_CRYPTO: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("eip155:1/erc20:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", "USDC"), // USDC on Ethereum
        ("eip155:137/erc20:0x2791bca1f2de4661ed88a30c99a7a9449aa84174", "USDC"), // USDC on Polygon
        ("eip155:8453/erc20:0x833589fcd6edb6e08f4c7c32d4f71b54bda02913", "USDC"), // USDC on Base
        ("eip155:42161/erc20:0xaf88d065e77c8cc2239327c5edb3a432268e5831", "USDC"), // USDC on Arbitrum
        ("eip155:1/slip44:60", "ETH"),  // Native ETH
        ("solana:4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ/slip44:501", "SOL"), // Native SOL
    ])
});

// CAIP-2 chain ID mappings to Binance networks
static CHAIN_ID_TO_BINANCE_NETWORK: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("eip155:1", "ETH"),       // Ethereum
        ("eip155:137", "MATIC"),   // Polygon
        ("eip155:8453", "BASE"),   // Base
        ("eip155:42161", "ARBITRUM"),   // Arbitrum
        ("eip155:10", "OPTIMISM"),   // Optimism
        ("solana:4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ", "SOL"), // Solana
    ])
});

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreOrderRequest {
    /// The unique order id from the partner side. Supports only letters and numbers.
    pub external_order_id: String,
    
    /// Fiat currency. If not specified, Binance Connect will automatically select/recommend a default fiat currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fiat_currency: Option<String>,
    
    /// Crypto currency. If not specified, Binance Connect will automatically select/recommend a default crypto currency.
    /// Required for SEND_PRIMARY.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_currency: Option<String>,
    
    /// Specify whether the requested amount is in fiat:1 or crypto:2
    //pub amount_type: i32, // TODO: Unsupported by Binance ATM
    /// Requested amount. Fraction is 8
    //pub requested_amount: String,
    pub fiat_amount: String,

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
pub struct Customization {

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
        Some("https://cryptologos.cc/logos/binance-coin-bnb-logo.png")
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
            (
                Some(client_id), 
                Some(key), 
                Some(token), 
                Some(host)) => Ok(
                    (client_id, key, token, host)
                ),
            _ => Err(ExchangeError::ConfigurationError("Exchange is not available".to_string())),
        }
    }

    fn generate_signature(
        &self,
        body: &str,
        timestamp: u64,
        private_key: &str,
    ) -> Result<String, ExchangeError> {
        let key_bytes = STANDARD.decode(private_key)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to decode private key: {}", e)))?;

        let pkey = PKey::private_key_from_pkcs8(&key_bytes)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to parse private key: {}", e)))?;

        let data_to_sign = if body.is_empty() || body == "{}" {
            timestamp.to_string()
        } else {
            format!("{}{}", body, timestamp)
        };
        debug!("Data to sign: {}", data_to_sign);

        let mut signer = Signer::new(MessageDigest::sha256(), &pkey)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to create signer: {}", e)))?;

        signer.update(data_to_sign.as_bytes()).map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to update signer: {}", e)))?;
        let signature = signer.sign_to_vec().map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to sign data: {}", e)))?;
        
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
        
        let body = serde_json::to_string(payload)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to serialize request body: {}", e)))?;
        
        let signature = self.generate_signature(&body, timestamp, &private_key)?;
        
        let url = format!("{}{}", host, path);
        
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
        
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(ExchangeError::InternalError(format!(
                "Binance API request failed with status: {}, body: {}",
                status, error_body
            )));
        }
    
        let parsed_response: BinanceResponse<R> = response
            .json()
            .await
            .map_err(|e| {
                debug!("Unable to parse Binance response: {}", e);
                ExchangeError::InternalError(format!("Failed to parse Binance response: {}", e))
            })?;
        debug!("Parsed response: {:?}", parsed_response);
        if let Some(success) = parsed_response.success {
            if !success {
                return Err(ExchangeError::InternalError(format!(
                    "Binance API request failed with code: {}, message: {}",
                    parsed_response.code, parsed_response.message.unwrap_or_default()
                )));
            }
        }

        parsed_response.data.ok_or_else(|| 
            ExchangeError::InternalError("No data returned from Binance".to_string())
        )
    }

    pub fn map_asset_to_binance_format(
        &self,
        asset: &Caip19Asset,
    ) -> Result<(String, String), ExchangeError> {
        let full_caip19 = asset.to_string();
        let chain_id = asset.chain_id().to_string();
        
        let crypto = CAIP19_TO_BINANCE_CRYPTO
            .get(full_caip19.as_str())
            .ok_or_else(|| ExchangeError::ValidationError(
                format!("Unsupported asset: {}", full_caip19)
            ))?
            .to_string();
        
        let network = CHAIN_ID_TO_BINANCE_NETWORK
            .get(chain_id.as_str())
            .ok_or_else(|| ExchangeError::ValidationError(
                format!("Unsupported chain ID: {}", chain_id)
            ))?
            .to_string();
        
        Ok((crypto, network))
    }

    pub async fn get_buy_url(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyUrlParams,
    ) -> Result<String, ExchangeError> {

        let (crypto_currency, network) = self.map_asset_to_binance_format(&params.asset).map_err(|e| ExchangeError::ValidationError(e.to_string()))?;
        
        let is_supported = self.is_payment_method_supported(
            &state,
            DEFAULT_PAYMENT_METHOD_CODE,
            DEFAULT_PAYMENT_METHOD_SUB_CODE,
            &params.amount.to_string(),
            &crypto_currency
        ).await?;
        
        if !is_supported {
            return Err(ExchangeError::ValidationError(
                "Selected payment method is not supported for this transaction".to_string()
            ));
        }

        let order_id = Uuid::new_v4().to_string().replace("-", "");

        let request = PreOrderRequest {
            external_order_id: order_id,
            fiat_currency: Some(DEFAULT_FIAT_CURRENCY.to_string()),
            crypto_currency: Some(crypto_currency),
            fiat_amount: params.amount.to_string(), // USING CRYPTO AMOUNT AS FIAT AMOUNT - This is indended atm
            pay_method_code: Some(DEFAULT_PAYMENT_METHOD_CODE.to_string()),
            pay_method_sub_code: Some(DEFAULT_PAYMENT_METHOD_SUB_CODE.to_string()),
            network,
            address: params.recipient,
            memo: None,
            redirect_url: None,
            fail_redirect_url: None,
            redirect_deep_link: None,
            fail_redirect_deep_link: None,
            client_ip: None,
            client_type: None,
            customization: None,
        };

        let url = self.create_pre_order(&state, request).await?;
        Ok(url)
    }

    pub async fn create_pre_order(
        &self,
        state: &Arc<AppState>,
        request: PreOrderRequest,
    ) -> Result<String, ExchangeError> {
        let data: PreOrderResponseData = self.send_post_request(state, PRE_ORDER_PATH, &request).await?;
        Ok(data.link)
    }


    pub async fn is_payment_method_supported(
        &self,
        state: &Arc<AppState>,
        payment_method_code: &str,
        payment_method_sub_code: &str,
        amount: &str,
        crypto_currency: &str,
    ) -> Result<bool, ExchangeError> {
        let request = PaymentMethodListRequest {
            fiat_currency: DEFAULT_FIAT_CURRENCY.to_string(),
            crypto_currency: crypto_currency.to_string(),
            total_amount: amount.to_string(),
            amount_type: AmountType::Crypto as usize,
        };
        
        let data: PaymentMethodListResponseData = self.send_post_request(state, PAYMENT_METHOD_LIST_PATH, &request).await?;

        let method = data.payment_methods
            .iter()
            .find(|method| 
                method.pay_method_code.as_deref() == Some(payment_method_code) && 
                method.pay_method_sub_code.as_deref() == Some(payment_method_sub_code))
            .ok_or_else(|| ExchangeError::ValidationError("Payment method is not supported".to_string()))?;

        let amount_value = amount.parse::<f64>()
            .map_err(|_| ExchangeError::ValidationError("Invalid amount format".to_string()))?;
        
        if let Some(min_limit_str) = &method.crypto_min_limit {
            let min_limit = min_limit_str.parse::<f64>()
                .map_err(|_| ExchangeError::ValidationError("Invalid min limit format".to_string()))?;
            
            if amount_value < min_limit {
                return Err(ExchangeError::ValidationError(
                    format!("Amount is below minimum limit of {}", min_limit_str)
                ));
            }
        }
        if let Some(max_limit_str) = &method.crypto_max_limit {
            let max_limit = max_limit_str.parse::<f64>()
                .map_err(|_| ExchangeError::ValidationError("Invalid max limit format".to_string()))?;
            
            if amount_value > max_limit {
                return Err(ExchangeError::ValidationError(
                    format!("Amount exceeds maximum limit of {}", max_limit_str)
                ));
            }
        }
        
        Ok(true)
    }

    
}
