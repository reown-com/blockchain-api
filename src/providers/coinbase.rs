use {
    super::{HistoryProvider, OnRampProvider},
    crate::{
        error::{RpcError, RpcResult},
        handlers::{
            generators::onrampurl::OnRampURLRequest,
            history::{
                HistoryQueryParams, HistoryResponseBody, HistoryTransaction,
                HistoryTransactionFungibleInfo, HistoryTransactionMetadata,
                HistoryTransactionTransfer, HistoryTransactionTransferQuantity,
            },
            onramp::{
                options::{OnRampBuyOptionsParams, OnRampBuyOptionsResponse},
                quotes::{OnRampBuyQuotesParams, OnRampBuyQuotesResponse},
            },
            wallet::exchanges::ExchangeError,
        },
        providers::{ProviderKind, TokenMetadataCacheProvider},
        utils::crypto::ChainId,
        Metrics,
    },
    async_trait::async_trait,
    base64::engine::general_purpose::STANDARD,
    base64::prelude::*,
    ed25519_dalek::{Signer, SigningKey},
    rand::RngCore,
    serde::{Deserialize, Serialize},
    std::{
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    },
    tracing::log::error,
    url::Url,
};

const CB_PAY_HOST: &str = "https://pay.coinbase.com";
const CB_PAY_PATH: &str = "/buy/select-asset";

#[derive(Debug)]
pub struct CoinbaseProvider {
    pub provider_kind: ProviderKind,
    pub api_key_id: String,
    pub api_key_secret: String,
    pub base_api_url: String,
    pub http_client: reqwest::Client,
}

impl CoinbaseProvider {
    pub fn new(api_key_id: String, api_key_secret: String, base_api_url: String) -> Self {
        Self {
            provider_kind: ProviderKind::Coinbase,
            api_key_id,
            api_key_secret,
            base_api_url,
            http_client: reqwest::Client::new(),
        }
    }

    async fn send_post_request<T>(
        &self,
        url: Url,
        params: &T,
        jwt_token: &str,
    ) -> Result<reqwest::Response, reqwest::Error>
    where
        T: Serialize,
    {
        self.http_client
            .post(url)
            .json(&params)
            .header("Authorization", format!("Bearer {jwt_token}"))
            .header("Content-Type", "application/json")
            .send()
            .await
    }

    async fn send_get_request(
        &self,
        url: Url,
        jwt_token: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("Authorization", format!("Bearer {jwt_token}"))
            .header("Content-Type", "application/json")
            .send()
            .await
    }

    async fn send_jwt_post_request<T>(
        &self,
        url: Url,
        params: &T,
        jwt_token: &str,
    ) -> Result<reqwest::Response, reqwest::Error>
    where
        T: Serialize,
    {
        self.http_client
            .post(url)
            .json(&params)
            .header("Authorization", format!("Bearer {jwt_token}"))
            .header("Content-Type", "application/json")
            .send()
            .await
    }

    pub async fn generate_session_token(
        &self,
        key_name: &str,
        key_secret: &str,
        addresses: Vec<SessionTokenAddress>,
        assets: Vec<String>,
        metrics: Arc<Metrics>,
    ) -> Result<String, ExchangeError> {
        let base = "https://api.developer.coinbase.com/onramp/v1/token";
        let url = Url::parse(base)
            .map_err(|_| ExchangeError::InternalError("Failed to parse URL".to_string()))?;

        // Generate JWT token
        let jwt_token = generate_jwt_key(
            key_name,
            key_secret,
            "POST",
            "api.developer.coinbase.com",
            "/onramp/v1/token",
        )?;

        let request_body = SessionTokenRequest { addresses, assets };

        let latency_start = SystemTime::now();
        let response = self
            .send_jwt_post_request(url, &request_body, &jwt_token)
            .await
            .map_err(|e| ExchangeError::InternalError(format!("Request failed: {e}")))?;

        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("session_token".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Coinbase session token response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(ExchangeError::InternalError(format!(
                "Session token request failed with status: {}",
                response.status()
            )));
        }

        let session_response: SessionTokenResponse = response
            .json()
            .await
            .map_err(|e| ExchangeError::InternalError(format!("Failed to parse response: {e}")))?;

        Ok(session_response.token)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CoinbaseResponseBody {
    pub transactions: Vec<CoinbaseTransaction>,
    pub next_page_key: Option<String>,
    pub total_count: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CoinbaseTransaction {
    pub status: String,
    pub transaction_id: String,
    pub tx_hash: String,
    pub created_at: String,
    pub purchase_network: String,
    pub purchase_amount: CoinbasePurchaseAmount,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CoinbasePurchaseAmount {
    pub value: String,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTokenRequest {
    pub addresses: Vec<SessionTokenAddress>,
    pub assets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTokenAddress {
    pub address: String,
    pub blockchains: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTokenResponse {
    pub token: String,
}

#[async_trait]
impl HistoryProvider for CoinbaseProvider {
    async fn get_transactions(
        &self,
        address: String,
        params: HistoryQueryParams,
        _metadata_cache: &Arc<dyn TokenMetadataCacheProvider>,
        metrics: Arc<Metrics>,
    ) -> RpcResult<HistoryResponseBody> {
        let base = format!("{}/buy/user/{}/transactions", &self.base_api_url, &address);

        let mut url = Url::parse(&base).map_err(|_| RpcError::HistoryParseCursorError)?;
        url.query_pairs_mut().append_pair("page_size", "50");

        if let Some(cursor) = params.cursor {
            url.query_pairs_mut().append_pair("page_key", &cursor);
        }

        let latency_start = SystemTime::now();
        let jwt_token = generate_jwt_key(
            &self.api_key_id,
            &self.api_key_secret,
            "GET",
            "api.developer.coinbase.com",
            "/buy/user/{}/transactions",
        )
        .map_err(|_| RpcError::TransactionProviderError)?;
        let response = self.send_get_request(url, &jwt_token).await?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("transactions".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Coinbase transactions response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::TransactionProviderError);
        }

        let body = response.json::<CoinbaseResponseBody>().await?;

        let transactions = body
            .transactions
            .into_iter()
            .map(|f| HistoryTransaction {
                id: f.transaction_id,
                metadata: HistoryTransactionMetadata {
                    operation_type: "buy".to_string(),
                    hash: f.tx_hash,
                    mined_at: f.created_at,
                    nonce: 1, // TODO: get nonce from somewhere
                    sent_from: "Coinbase".to_string(),
                    sent_to: address.clone(),
                    status: f.status,
                    application: None,
                    chain: ChainId::to_caip2(&f.purchase_network),
                },
                transfers: Some(vec![HistoryTransactionTransfer {
                    fungible_info: Some(HistoryTransactionFungibleInfo {
                        name: Some(f.purchase_amount.currency.clone()),
                        symbol: Some(f.purchase_amount.currency),
                        icon: None,
                    }),
                    direction: "in".to_string(),
                    quantity: HistoryTransactionTransferQuantity {
                        numeric: f.purchase_amount.value,
                    },
                    nft_info: None,
                    value: None,
                    price: None,
                }]),
            })
            .collect();

        Ok(HistoryResponseBody {
            data: transactions,
            next: body.next_page_key,
        })
    }

    fn provider_kind(&self) -> ProviderKind {
        self.provider_kind
    }
}

#[async_trait]
impl OnRampProvider for CoinbaseProvider {
    #[tracing::instrument(skip(self), fields(provider = "Coinbase"), level = "debug")]
    async fn get_buy_options(
        &self,
        params: OnRampBuyOptionsParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<OnRampBuyOptionsResponse> {
        let base = format!("{}/buy/options", &self.base_api_url);
        let mut url = Url::parse(&base).map_err(|_| RpcError::OnRampParseURLError)?;
        url.query_pairs_mut()
            .append_pair("country", &params.country);
        if let Some(subdivision) = params.subdivision {
            url.query_pairs_mut()
                .append_pair("subdivision", &subdivision);
        }

        let latency_start = SystemTime::now();
        let jwt_token = generate_jwt_key(
            &self.api_key_id,
            &self.api_key_secret,
            "GET",
            "api.developer.coinbase.com",
            "/buy/options",
        )
        .map_err(|_| RpcError::OnRampProviderError)?;
        let response = self.send_get_request(url, &jwt_token).await?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("buy_options".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on CoinBase buy options response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::OnRampProviderError);
        }

        Ok(response.json::<OnRampBuyOptionsResponse>().await?)
    }

    async fn get_buy_quotes(
        &self,
        params: OnRampBuyQuotesParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<OnRampBuyQuotesResponse> {
        let base = format!("{}/buy/quote", &self.base_api_url);
        let url = Url::parse(&base).map_err(|_| RpcError::OnRampParseURLError)?;

        let latency_start = SystemTime::now();
        let jwt_token = generate_jwt_key(
            &self.api_key_id,
            &self.api_key_secret,
            "POST",
            "api.developer.coinbase.com",
            "/buy/quote",
        )
        .map_err(|_| RpcError::OnRampProviderError)?;
        let response = self.send_post_request(url, &params, &jwt_token).await?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("buy_quote".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on CoinBase buy quotes response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::OnRampProviderError);
        }

        Ok(response.json::<OnRampBuyQuotesResponse>().await?)
    }

    async fn generate_on_ramp_url(
        &self,
        parameters: OnRampURLRequest,
        metrics: Arc<Metrics>,
    ) -> Result<String, anyhow::Error> {
        let mut url = Url::parse(CB_PAY_HOST)?;
        url.set_path(CB_PAY_PATH);

        let session_token_addresses: Vec<SessionTokenAddress> = parameters
            .destination_wallets
            .iter()
            .filter_map(|wallet| {
                wallet
                    .blockchains
                    .as_ref()
                    .map(|chains| SessionTokenAddress {
                        address: wallet.address.clone(),
                        blockchains: chains.clone(),
                    })
            })
            .collect();

        let assets = parameters
            .destination_wallets
            .iter()
            .filter_map(|wallet| wallet.blockchains.clone())
            .flatten()
            .collect();

        let token = self
            .generate_session_token(
                &self.api_key_id,
                &self.api_key_secret,
                session_token_addresses,
                assets,
                metrics,
            )
            .await?;

        // Required parameters
        url.query_pairs_mut().append_pair("sessionToken", &token);
        url.query_pairs_mut().append_pair(
            "destinationWallets",
            &serde_json::to_string(&parameters.destination_wallets)?,
        );
        url.query_pairs_mut()
            .append_pair("partnerUserId", &parameters.partner_user_id);

        // Optional parameters
        if let Some(default_network) = parameters.default_network {
            url.query_pairs_mut()
                .append_pair("defaultNetwork", &default_network);
        }
        if let Some(preset_crypto_amount) = parameters.preset_crypto_amount {
            url.query_pairs_mut()
                .append_pair("presetCryptoAmount", &preset_crypto_amount.to_string());
        }
        if let Some(preset_fiat_amount) = parameters.preset_fiat_amount {
            url.query_pairs_mut()
                .append_pair("presetFiatAmount", &preset_fiat_amount.to_string());
        }
        if let Some(default_experience) = parameters.default_experience {
            url.query_pairs_mut()
                .append_pair("defaultExperience", &default_experience.to_string());
        }
        if let Some(handling_requested_urls) = parameters.handling_requested_urls {
            url.query_pairs_mut().append_pair(
                "handlingRequestedUrls",
                &handling_requested_urls.to_string(),
            );
        }

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

/// Generate Coinbase JWT key
/// https://docs.cdp.coinbase.com/api-reference/v2/authentication#generate-bearer-token-jwt-and-export
pub fn generate_jwt_key(
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
