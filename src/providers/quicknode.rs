use {
    super::{
        Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory, RpcQueryParams,
        RpcWsProvider,
    },
    crate::{
        env::QuicknodeConfig,
        error::{RpcError, RpcResult},
        json_rpc::JsonRpcRequest,
        ws,
    },
    async_trait::async_trait,
    axum::{
        extract::ws::WebSocketUpgrade,
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::http,
    serde::Serialize,
    std::collections::HashMap,
    tracing::debug,
    wc::metrics::{future_metrics, FutureExt},
};

#[derive(Debug, Serialize)]
struct BroadcastTransactionRequest {
    #[serde(rename = "txID")]
    pub txid: String,
    pub visible: bool,
    pub raw_data: serde_json::Value,
    pub raw_data_hex: String,
    pub signature: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TronApiResult {
    pub result: serde_json::Value,
}

const TRON_CHAIN_ID: &str = "tron:0x2b6653dc";

/// The method name for the tron broadcast transaction wrapper method
const TRON_BROADCAST_TRANSACTION_METHOD: &str = "tron_broadcastTransaction";

#[derive(Debug)]
pub struct QuicknodeProvider {
    pub client: reqwest::Client,
    pub supported_chains: HashMap<String, String>,
    pub chain_subdomains: HashMap<String, String>,
}

impl Provider for QuicknodeProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Quicknode
    }
}

impl QuicknodeProvider {
    /// Helper function to wrap response body in JSON-RPC format
    /// that uses the `result` field to wrap the non-JSON-RPC response body.
    fn wrap_response_in_result(&self, body: &[u8]) -> Result<Vec<u8>, RpcError> {
        let original_result = match serde_json::from_slice::<serde_json::Value>(body) {
            Ok(value) => value,
            Err(e) => {
                return Err(RpcError::InvalidParameter(format!(
                    "Failed to deserialize Quicknode non-JSON-RPC response: {e}"
                )));
            }
        };
        let wrapped_response = TronApiResult {
            result: original_result,
        };
        serde_json::to_vec(&wrapped_response).map_err(|e| {
            RpcError::InvalidParameter(format!(
                "Failed to serialize wrapped Quicknode response: {e}"
            ))
        })
    }

    async fn handle_tron_broadcast_transaction(
        &self,
        params_value: serde_json::Value,
    ) -> RpcResult<Response> {
        // params array must be 5 elements for this method
        let params = params_value.as_array().ok_or(RpcError::InvalidParameter(
            "Params must be an array for tron_broadcastTransaction method".to_string(),
        ))?;
        if params.len() != 5 {
            return Err(RpcError::InvalidParameter(
                "Params array must be 5 elements for tron_broadcastTransaction method".to_string(),
            ));
        }
        let txid = params[0]
            .as_str()
            .ok_or_else(|| RpcError::InvalidParameter("TxID is not a string".to_string()))?;
        let visible = params[1].as_bool().unwrap_or(false);
        let raw_data = params[2].clone();
        let raw_data_hex = params[3].as_str().ok_or_else(|| {
            RpcError::InvalidParameter("Raw data hex is not a string".to_string())
        })?;
        // Signature can be an array or a JSON-encoded string array
        let signature_owned: Vec<String> = if let Some(arr) = params[4].as_array() {
            arr.iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect()
        } else if let Some(s) = params[4].as_str() {
            let trimmed = s.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                serde_json::from_str::<Vec<String>>(trimmed).map_err(|e| {
                    RpcError::InvalidParameter(format!(
                        "Signature must be a JSON array of strings when provided as a string: {e}"
                    ))
                })?
            } else {
                vec![s.to_string()]
            }
        } else {
            return Err(RpcError::InvalidParameter(
                "Signature must be an array of strings or JSON-encoded array string".to_string(),
            ));
        };
        let signature: Vec<&str> = signature_owned.iter().map(|s| s.as_str()).collect();

        self.tron_broadcast_transaction(txid, visible, raw_data, raw_data_hex, signature)
            .await
    }

    // Send request to the Tron broadcast transaction `/wallet/broadcasttransaction` API endpoint
    async fn tron_broadcast_transaction(
        &self,
        txid: &str,
        visible: bool,
        raw_data: serde_json::Value,
        raw_data_hex: &str,
        signature: Vec<&str>,
    ) -> RpcResult<Response> {
        let token = &self
            .supported_chains
            .get(TRON_CHAIN_ID)
            .ok_or(RpcError::ChainNotFound)?;

        let chain_subdomain =
            self.chain_subdomains
                .get(TRON_CHAIN_ID)
                .ok_or(RpcError::InvalidConfiguration(
                    "Quicknode subdomain not found for Tron chain".to_string(),
                ))?;

        let uri =
            format!("https://{chain_subdomain}.quiknode.pro/{token}/wallet/broadcasttransaction");

        let transactions_request = serde_json::to_string(&BroadcastTransactionRequest {
            txid: txid.to_string(),
            visible,
            raw_data,
            raw_data_hex: raw_data_hex.to_string(),
            signature: signature.iter().map(|s| s.to_string()).collect(),
        })
        .map_err(|e| RpcError::InvalidParameter(format!("Failed to serialize transaction: {e}")))?;

        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(transactions_request)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                 Quicknode transactions: {response:?}"
                );
            }
        }

        let wrapped_body = self.wrap_response_in_result(&body)?;
        let mut response = (status, wrapped_body).into_response();
        response
            .headers_mut()
            .insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok(response)
    }
}

#[async_trait]
impl RateLimited for QuicknodeProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for QuicknodeProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let token = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        // Get the chain subdomain
        let chain_subdomain =
            self.chain_subdomains
                .get(chain_id)
                .ok_or(RpcError::InvalidConfiguration(format!(
                    "Quicknode subdomain not found for chainId: {chain_id}"
                )))?;

        // Check for the tron broadcast transaction method exclusion
        let json_rpc_request: JsonRpcRequest = serde_json::from_slice(&body)
            .map_err(|_| RpcError::InvalidParameter("Invalid JSON-RPC schema provided".into()))?;
        let method = json_rpc_request.method.to_string();
        let params = json_rpc_request.params;
        // Handle the tron broadcast transaction wrapped method and pass the parameters form an array
        if method == TRON_BROADCAST_TRANSACTION_METHOD {
            return self.handle_tron_broadcast_transaction(params).await;
        }

        // Add /jsonrpc prefix for the Tron and /jsonRPC prefix for the Ton
        let uri = match chain_id {
            "tron:0x2b6653dc" => format!("https://{chain_subdomain}.quiknode.pro/{token}/jsonrpc"),
            "ton:mainnet" => format!("https://{chain_subdomain}.quiknode.pro/{token}/jsonRPC"),
            _ => format!("https://{chain_subdomain}.quiknode.pro/{token}"),
        };

        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;
        let mut response = (status, body).into_response();
        response
            .headers_mut()
            .insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok(response)
    }
}

impl RpcProviderFactory<QuicknodeConfig> for QuicknodeProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &QuicknodeConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        QuicknodeProvider {
            client: forward_proxy_client,
            supported_chains,
            chain_subdomains: provider_config.chain_subdomains.clone(),
        }
    }
}

#[derive(Debug)]
pub struct QuicknodeWsProvider {
    pub supported_chains: HashMap<String, String>,
    pub chain_subdomains: HashMap<String, String>,
}

impl Provider for QuicknodeWsProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Quicknode
    }
}

#[async_trait]
impl RpcWsProvider for QuicknodeWsProvider {
    #[tracing::instrument(skip_all, fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response> {
        let chain_id = &query_params.chain_id;
        let project_id = query_params.project_id;
        let token = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let chain_subdomain =
            self.chain_subdomains
                .get(chain_id)
                .ok_or(RpcError::InvalidConfiguration(format!(
                    "Quicknode wss subdomain not found for chainId: {chain_id}"
                )))?;
        let uri = format!("wss://{chain_subdomain}.quiknode.pro/{token}");
        let (websocket_provider, _) = async_tungstenite::tokio::connect_async(uri)
            .await
            .map_err(|e| RpcError::WebSocketError(e.to_string()))?;

        Ok(ws.on_upgrade(move |socket| {
            ws::proxy(project_id, socket, websocket_provider)
                .with_metrics(future_metrics!("ws_proxy_task", "name" => "quicknode"))
        }))
    }
}

#[async_trait]
impl RateLimited for QuicknodeWsProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

impl RpcProviderFactory<QuicknodeConfig> for QuicknodeWsProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &QuicknodeConfig) -> Self {
        let supported_chains: HashMap<String, String> = provider_config
            .supported_ws_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();
        let chain_subdomains = provider_config.chain_subdomains.clone();

        QuicknodeWsProvider {
            supported_chains,
            chain_subdomains,
        }
    }
}
