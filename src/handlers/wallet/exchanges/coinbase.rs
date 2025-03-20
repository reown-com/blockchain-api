use crate::handlers::wallet::exchanges::{ExchangeError, ExchangeProvider};
use crate::state::AppState;
use axum::extract::State;
use base64::engine::general_purpose::STANDARD;
use base64::prelude::*;

use ed25519_dalek::{Signer, SigningKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

const COINBASE_API_HOST: &str = "api.developer.coinbase.com";

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
            .header("Authorization", format!("Bearer {}", jwt_key))
            .send()
            .await
            .map_err(|e| ExchangeError::InternalError(e.to_string()))?;

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
        let (pub_key, priv_key) = self.get_api_credentials(state)?;

        let jwt_key =
            generate_coinbase_jwt_key(&pub_key, &priv_key, "POST", COINBASE_API_HOST, path)?;

        let url = format!("https://{}{}", COINBASE_API_HOST, path);

        let res = state
            .http_client
            .post(url)
            .json(payload)
            .header("Authorization", format!("Bearer {}", jwt_key))
            .send()
            .await
            .map_err(|e| ExchangeError::InternalError(e.to_string()))?;

        Ok(res)
    }

    pub async fn get_buy_url(
        state: State<Arc<AppState>>,
        _asset: &str,
        _amount: &str,
    ) -> Result<String, ExchangeError> {
        let exchange = CoinbaseExchange;
        let path = "/onramp/v1/buy/config";

        let res = exchange.send_get_request(&state, path).await?;

        let status = res.status();

        if !status.is_success() {
            return Err(ExchangeError::InternalError(format!(
                "Request failed with status: {}",
                status
            )));
        }

        match res.text().await {
            Ok(body) => {
                info!("body: {:?}", body);
                Ok(body)
            }
            Err(e) => Err(ExchangeError::InternalError(e.to_string())),
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
