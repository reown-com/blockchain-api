use crate::handlers::wallet::exchanges::{ExchangeError, ExchangeProvider, GetBuyUrlParams};
use crate::state::AppState;
use axum::extract::State;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use base64::{Engine, engine::general_purpose::STANDARD};
use tracing::info;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::hash::MessageDigest;
pub struct BinanceExchange;

const PRE_ORDER_PATH: &str = "/papi/v1/ramp/connect/buy/pre-order";

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
    //pub amount_type: i32,

    
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

#[derive(Debug, Deserialize)]
struct PreOrderResponse {
    success: bool,
    code: String,
    message: String,
    data: Option<PreOrderResponseData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreOrderResponseData {
    link: String,
    link_expire_time: u64,
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

        // For empty bodies, we only sign the timestamp
        let data_to_sign = if body.is_empty() || body == "{}" {
            timestamp.to_string()
        } else {
            format!("{}{}", body, timestamp)
        };
        info!("Data to sign: {}", data_to_sign);

        let mut signer = Signer::new(MessageDigest::sha256(), &pkey)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to create signer: {}", e)))?;

        signer.update(data_to_sign.as_bytes()).map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to update signer: {}", e)))?;
        let signature = signer.sign_to_vec().map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to sign data: {}", e)))?;
        
        Ok(STANDARD.encode(&signature))
    }

    async fn send_get_request(
        &self,
        state: &Arc<AppState>,
        path: &str,
    ) -> Result<reqwest::Response, ExchangeError> {
        let (client_id, private_key, token, host) = self.get_api_credentials(state)?;
        
        // Get timestamp in milliseconds, matching Java's System.currentTimeMillis()
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| ExchangeError::GetPayUrlError("Failed to get current time".to_string()))?
            .as_millis() as u64;
        
        let body = "";
        
        let signature = self.generate_signature(body, timestamp, &private_key)?;
        
        let url = format!("{}{}", host, path);
        
        let res = state
            .http_client
            .get(url)
            .header("X-Tesla-ClientId", &client_id)
            .header("X-Tesla-Timestamp", timestamp.to_string())
            .header("X-Tesla-Signature", signature)
            .header("X-Tesla-SignAccessToken", token)
            .send()
            .await
            .map_err(|e| ExchangeError::GetPayUrlError(e.to_string()))?;
        Ok(res)
    }

    async fn send_post_request<T>(
        &self,
        state: &Arc<AppState>,
        path: &str,
        payload: &T,
    ) -> Result<reqwest::Response, ExchangeError>
    where
        T: Serialize,
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
        

        let res = state
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
        
        Ok(res)
    }

    pub async fn get_buy_url(
        state: State<Arc<AppState>>,
        params: GetBuyUrlParams,
    ) -> Result<String, ExchangeError> {
        let exchange = BinanceExchange;
        let request = PreOrderRequest {
            external_order_id: "1234567890".to_string(),
            fiat_currency: Some("USD".to_string()),
            crypto_currency: Some("USDC".to_string()),
            fiat_amount: "20".to_string(),
            pay_method_code: Some("BUY_WALLET".to_string()),
            pay_method_sub_code: Some("Wallet".to_string()),
            network: "BASE".to_string(),
            address: "0x81D8C68Be5EcDC5f927eF020Da834AA57cc3Bd24".to_string(),
            memo: None,
            redirect_url: None,
            fail_redirect_url: None,
            redirect_deep_link: None,
            fail_redirect_deep_link: None,
            client_ip: None,
            client_type: None,
            customization: None,
        };
        let url = exchange.create_pre_order(&state, request).await?;
        Ok(url)
    }

    pub async fn create_pre_order(
        &self,
        state: &Arc<AppState>,
        request: PreOrderRequest,
    ) -> Result<String, ExchangeError> {
        
        let response = self.send_post_request(state, PRE_ORDER_PATH, &request).await?;
        
        let status = response.status();
        info!("Binance response status: {}", status);
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            info!("Binance API error: {}", error_body);
            return Err(ExchangeError::InternalError(format!(
                "Binance API request failed with status: {}, body: {}",
                status, error_body
            )));
        }
   
        let response: PreOrderResponse = response
        .json()
        .await
        .map_err(|e| {
            ExchangeError::InternalError(format!("Failed to parse Binance response: {}", e))
        })?;
        info!("Binance response: {:?}", response);
            
        if !response.success {
            return Err(ExchangeError::InternalError(format!(
                "Binance API request failed with code: {}, message: {}",
                response.code, response.message
            )));
        }

        if let Some(data) = response.data {
            Ok(data.link)
        } else {
            Err(ExchangeError::InternalError("No data returned from Binance".to_string()))
        }
    }
}
