use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::TrongridConfig,
        error::{RpcError, RpcResult},
        json_rpc::JsonRpcRequest,
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::http,
    serde::Serialize,
    std::collections::HashMap,
    tracing::debug,
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

const TRON_BROADCAST_TRANSACTION_METHOD: &str = "tron_broadcastTransaction";

#[derive(Debug)]
pub struct TrongridProvider {
    pub client: reqwest::Client,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for TrongridProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Trongrid
    }
}

impl TrongridProvider {
    fn wrap_response_in_result(&self, body: &[u8]) -> Result<Vec<u8>, RpcError> {
        let original_result = match serde_json::from_slice::<serde_json::Value>(body) {
            Ok(value) => value,
            Err(e) => {
                return Err(RpcError::InvalidParameter(format!(
                    "Failed to deserialize TronGrid non-JSON-RPC response: {e}"
                )));
            }
        };
        let wrapped_response = TronApiResult {
            result: original_result,
        };
        serde_json::to_vec(&wrapped_response).map_err(|e| {
            RpcError::InvalidParameter(format!(
                "Failed to serialize wrapped TronGrid response: {e}"
            ))
        })
    }

    async fn handle_tron_broadcast_transaction(
        &self,
        chain_id: &str,
        params_value: serde_json::Value,
    ) -> RpcResult<Response> {
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

        self.tron_broadcast_transaction(chain_id, txid, visible, raw_data, raw_data_hex, signature)
            .await
    }

    async fn tron_broadcast_transaction(
        &self,
        chain_id: &str,
        txid: &str,
        visible: bool,
        raw_data: serde_json::Value,
        raw_data_hex: &str,
        signature: Vec<&str>,
    ) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let base_url = uri.strip_suffix("/jsonrpc").unwrap_or(uri.as_str());
        let broadcast_uri = format!("{base_url}/wallet/broadcasttransaction");

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
            .post(broadcast_uri)
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
                     TronGrid transactions: {response:?}"
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
impl RateLimited for TrongridProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for TrongridProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let json_rpc_request: JsonRpcRequest = serde_json::from_slice(&body)
            .map_err(|_| RpcError::InvalidParameter("Invalid JSON-RPC schema provided".into()))?;
        let method = json_rpc_request.method.to_string();
        let params = json_rpc_request.params;

        if method == TRON_BROADCAST_TRANSACTION_METHOD {
            return self
                .handle_tron_broadcast_transaction(chain_id, params)
                .await;
        }

        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                     TronGrid: {response:?}"
                );
            }
        }

        let mut response = (status, body).into_response();
        response
            .headers_mut()
            .insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok(response)
    }
}

impl RpcProviderFactory<TrongridConfig> for TrongridProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &TrongridConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        TrongridProvider {
            client: forward_proxy_client,
            supported_chains,
        }
    }
}
