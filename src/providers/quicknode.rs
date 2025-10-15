use {
    super::{
        Provider,
        ProviderKind,
        RateLimited,
        RpcProvider,
        RpcProviderFactory,
        RpcQueryParams,
        RpcWsProvider,
    },
    crate::{
        env::QuicknodeConfig,
        error::{RpcError, RpcResult},
        json_rpc::{JsonRpcRequest, JsonRpcResult},
        ws,
    },
    async_trait::async_trait,
    axum::{
        extract::ws::WebSocketUpgrade,
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::http,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    tracing::debug,
    wc::metrics::{future_metrics, FutureExt},
};

#[derive(Debug, Serialize)]
struct TronBroadcastTransactionRequest {
    #[serde(rename = "txID")]
    pub txid: String,
    pub visible: bool,
    pub raw_data: serde_json::Value,
    pub raw_data_hex: String,
    pub signature: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TonSendBocRequest {
    pub boc: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TonApiErrorResponse {
    pub ok: bool,
    pub error: serde_json::Value,
    pub code: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TonApiSuccessResponse {
    pub ok: bool,
    pub result: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct TronApiResult {
    pub result: serde_json::Value,
}

const TRON_CHAIN_ID: &str = "tron:0x2b6653dc";

/// The method name for the TON sendBoc wrapper method
const TON_SEND_BOC_METHOD: &str = "ton_sendBoc";
const TON_CHAIN_ID: &str = "ton:mainnet";

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
        let visible = if let Some(b) = params[1].as_bool() {
            b
        } else if let Some(s) = params[1].as_str() {
            matches!(s.to_lowercase().as_str(), "true" | "1")
        } else {
            false
        };
        let raw_data = if let Some(s) = params[2].as_str() {
            serde_json::from_str(s).map_err(|_| {
                RpcError::InvalidParameter("Invalid JSON in raw_data parameter".to_string())
            })?
        } else {
            params[2].clone()
        };
        let raw_data_hex = params[3].as_str().ok_or_else(|| {
            RpcError::InvalidParameter("Raw data hex is not a string".to_string())
        })?;
        // Signature can be an array or a JSON-encoded string array
        let signature_owned: Vec<String> = if let Some(arr) = params[4].as_array() {
            arr.iter()
                .map(|v| {
                    v.as_str().ok_or_else(|| {
                        RpcError::InvalidParameter(
                            "Signature array contains non-string element".to_string(),
                        )
                    })
                })
                .collect::<Result<Vec<_>, RpcError>>()?
                .into_iter()
                .map(|s| s.to_string())
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

    async fn handle_ton_send_boc(
        &self,
        id: serde_json::Value,
        params_value: serde_json::Value,
    ) -> RpcResult<Response> {
        // params array must be 1 element: boc string
        let params = params_value.as_array().ok_or(RpcError::InvalidParameter(
            "Params must be an array for ton_sendBoc method".to_string(),
        ))?;
        if params.len() != 1 {
            return Err(RpcError::InvalidParameter(
                "Params array must be 1 element for ton_sendBoc method".to_string(),
            ));
        }
        let boc = params[0]
            .as_str()
            .ok_or_else(|| RpcError::InvalidParameter("boc is not a string".to_string()))?;

        self.ton_send_boc(id, boc).await
    }

    // Send request to the Tron broadcast transaction `/wallet/broadcasttransaction`
    // API endpoint
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

        let transactions_request = serde_json::to_string(&TronBroadcastTransactionRequest {
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

        // Handle the TON API error response which is HTTP 500 with the error structure
        // response
        if status == http::StatusCode::INTERNAL_SERVER_ERROR
            || status == http::StatusCode::SERVICE_UNAVAILABLE
        {
            if let Ok(error_response) = serde_json::from_slice::<TonApiErrorResponse>(&body) {
                return Err(RpcError::InvalidParameter(format!(
                    "TON API error: {error_response:?}"
                )));
            }
        }

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

    // Send request to the TON `/sendBoc` REST API endpoint
    async fn ton_send_boc(&self, id: serde_json::Value, boc: &str) -> RpcResult<Response> {
        let token = &self
            .supported_chains
            .get(TON_CHAIN_ID)
            .ok_or(RpcError::ChainNotFound)?;

        let chain_subdomain =
            self.chain_subdomains
                .get(TON_CHAIN_ID)
                .ok_or(RpcError::InvalidConfiguration(
                    "Quicknode subdomain not found for TON chain".to_string(),
                ))?;

        let uri = format!("https://{chain_subdomain}.quiknode.pro/{token}/sendBoc");

        let body = serde_json::to_vec(&TonSendBocRequest {
            boc: boc.to_string(),
        })
        .map_err(|e| {
            RpcError::InvalidParameter(format!("Failed to serialize TON sendBoc body: {e}"))
        })?;

        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        // If provider responded with a TON-shaped error body on server error status,
        // convert it to a Bad Request for the RPC layer; otherwise continue.
        if status == http::StatusCode::INTERNAL_SERVER_ERROR
            || status == http::StatusCode::SERVICE_UNAVAILABLE
        {
            if let Ok(error_response) = serde_json::from_slice::<TonApiErrorResponse>(&body) {
                // Return a JSON-RPC error envelope with the same request id
                let error_json = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": error_response.code,
                        "message": error_response.error,
                    }
                });
                let body = serde_json::to_vec(&error_json)?;
                // to allow a proper error handling by the RPC client we need to
                // return it as an HTTP 200 response with the error body
                let mut response = (http::StatusCode::OK, body).into_response();
                response
                    .headers_mut()
                    .insert("Content-Type", HeaderValue::from_static("application/json"));
                return Ok(response);
            }
        }

        // Wrap provider { ok, result } into JSON-RPC success envelope
        if let Ok(success_response) = serde_json::from_slice::<TonApiSuccessResponse>(&body) {
            let json = JsonRpcResult::new(id, success_response.result);
            let body = serde_json::to_vec(&json)?;
            let mut response = (http::StatusCode::OK, body).into_response();
            response
                .headers_mut()
                .insert("Content-Type", HeaderValue::from_static("application/json"));
            return Ok(response);
        }

        // Fallback: return the original body with the provider's status
        let mut response = (status, body).into_response();
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
        let id = json_rpc_request.id;
        let params = json_rpc_request.params;
        // Handle the tron broadcast transaction wrapped method and pass the parameters
        // form an array
        if method == TRON_BROADCAST_TRANSACTION_METHOD {
            return self.handle_tron_broadcast_transaction(params).await;
        }
        // Handle the TON sendBoc wrapped method and pass the parameters from an array
        if method == TON_SEND_BOC_METHOD {
            return self.handle_ton_send_boc(id, params).await;
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
