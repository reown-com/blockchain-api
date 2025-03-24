use crate::handlers::wallet::exchanges::{ExchangeError, ExchangeProvider};
use crate::state::AppState;
use axum::extract::State;
use std::sync::Arc;
use serde::Serialize;
use rsa::{
    RsaPrivateKey, 
    pkcs8::DecodePrivateKey,
    Pkcs1v15Sign
};
use base64::{Engine, engine::general_purpose::STANDARD};
use tracing::info;
use sha256;
use hex;

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

        let private_key = RsaPrivateKey::from_pkcs8_der(&key_bytes)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to parse private key: {}", e)))?;

        let data_to_sign = format!("{}{}", body, timestamp);
        
        let hashed_data = sha256::digest(data_to_sign.as_bytes());
        let hashed_bytes = hex::decode(hashed_data)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to decode hash: {}", e)))?;
        
        let signing_key = private_key;
        let signature = signing_key.sign(Pkcs1v15Sign::new_unprefixed(), &hashed_bytes)
            .map_err(|e| ExchangeError::GetPayUrlError(format!("Failed to sign data: {}", e)))?;
        
        Ok(STANDARD.encode(&signature))
    }

    async fn send_get_request(
        &self,
        state: &Arc<AppState>,
        path: &str,
    ) -> Result<reqwest::Response, ExchangeError> {
        let (client_id, private_key, token, host) = self.get_api_credentials(state)?;
        
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
        asset: &str,
        amount: &str,
    ) -> Result<String, ExchangeError> {
        let exchange = BinanceExchange;
        
        // Define request payload for generating buy URL
        #[derive(Debug, Serialize)]
        struct BuyQuoteRequest {
            asset: String,
            amount: String,
            fiat_currency: String,
        }
        
        let request = BuyQuoteRequest {
            asset: asset.to_string(),
            amount: amount.to_string(),
            fiat_currency: "USD".to_string(),
        };
        
        // Path for the buy quote endpoint
        const BUY_QUOTE_PATH: &str = "/api/v1/buy/quote";
        
        // Make the API request
        let response = exchange.send_post_request(&state, BUY_QUOTE_PATH, &request).await?;
        
        // Check if the request was successful
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            info!("Binance API error: {}", error_body);
            return Err(ExchangeError::InternalError(format!(
                "Binance API request failed with status: {}, body: {}",
                status, error_body
            )));
        }
        
        // Parse the response
        #[derive(Debug, serde::Deserialize)]
        struct BuyQuoteResponse {
            quote_id: String,
            payment_url: String,
        }
        
        let quote: BuyQuoteResponse = response
            .json()
            .await
            .map_err(|e| {
                ExchangeError::InternalError(format!("Failed to parse Binance response: {}", e))
            })?;
        
        // Return the payment URL
        Ok(quote.payment_url)
    }
}
