use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::HiroConfig,
        error::{RpcError, RpcResult},
        json_rpc::JsonRpcRequest,
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::http,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    tracing::debug,
};

#[derive(Debug)]
pub struct HiroProvider {
    pub client: reqwest::Client,
    pub supported_chains: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportedMethods {
    StacksTransactions,
    StacksAccounts,
    StacksExtendedNonces,
    StacksTransferFees,
    HiroFeesTransaction,
}

#[derive(Debug, Serialize)]
pub struct TransactionsRequest {
    pub tx: String,
}

#[derive(Debug, Serialize)]
pub struct FeesTransactionRequest {
    pub transaction_payload: String,
}

#[derive(Debug, Serialize)]
pub struct HiroResult {
    pub result: serde_json::Value,
}

impl Provider for HiroProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Hiro
    }
}

#[async_trait]
impl RateLimited for HiroProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

impl HiroProvider {
    /// Helper function to wrap response body in JSON-RPC format
    /// that uses the `result` field to wrap the response body.
    fn wrap_response_in_result(&self, body: &[u8]) -> Result<Vec<u8>, RpcError> {
        let original_result = match serde_json::from_slice::<serde_json::Value>(body) {
            Ok(value) => value,
            Err(e) => {
                return Err(RpcError::InvalidParameter(format!(
                    "Failed to deserialize Hiro response: {e}"
                )));
            }
        };
        let wrapped_response = HiroResult {
            result: original_result,
        };
        serde_json::to_vec(&wrapped_response).map_err(|e| {
            RpcError::InvalidParameter(format!("Failed to serialize wrapped Hiro response: {e}"))
        })
    }

    // Send request to the Stacks `/v2/transactions` endpoint
    async fn transactions(&self, chain_id: String, tx: String) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&chain_id)
            .ok_or(RpcError::ChainNotFound)?;
        let uri = format!("{}/v2/transactions", uri.trim_end_matches('/'));

        let stacks_transactions_request = serde_json::to_string(&TransactionsRequest { tx })
            .map_err(|e| {
                RpcError::InvalidParameter(format!("Failed to serialize transaction: {e}"))
            })?;

        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(stacks_transactions_request)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                     Hiro transactions: {response:?}"
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

    // Send request to the Stacks `/v2/accounts` endpoint
    async fn accounts(&self, chain_id: String, principal: String) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&chain_id)
            .ok_or(RpcError::ChainNotFound)?;
        let uri = format!("{}/v2/accounts/{}", uri.trim_end_matches('/'), principal);

        let response = self
            .client
            .get(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                     Hiro accounts: {response:?}"
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

    // Send request to the Hiro `/v2/fees/transaction` endpoint
    async fn fees_transaction(
        &self,
        chain_id: String,
        transaction_payload: String,
    ) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&chain_id)
            .ok_or(RpcError::ChainNotFound)?;
        let uri = format!("{}/v2/fees/transaction", uri.trim_end_matches('/'));

        let hiro_fees_transaction_request = serde_json::to_string(&FeesTransactionRequest {
            transaction_payload,
        })
        .map_err(|e| {
            RpcError::InvalidParameter(format!("Failed to serialize fees transaction: {e}"))
        })?;

        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(hiro_fees_transaction_request)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                     Hiro fees transaction: {response:?}"
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

    // Send request to the Stacks `/v2/fees/transfer` endpoint
    async fn transfer_fees(&self, chain_id: String) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&chain_id)
            .ok_or(RpcError::ChainNotFound)?;
        let uri = format!("{}/v2/fees/transfer", uri.trim_end_matches('/'));

        let response = self.client.get(uri).send().await?;
        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                     Stacks transfer fees: {response:?}"
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

    // Send request to the Stacks `/extended/v1/address/<principal>/nonces` endpoint
    async fn extended_nonces(&self, chain_id: String, principal: String) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&chain_id)
            .ok_or(RpcError::ChainNotFound)?;
        let uri = format!(
            "{}/extended/v1/address/{}/nonces",
            uri.trim_end_matches('/'),
            principal
        );

        let response = self
            .client
            .get(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                     Stacks extended nonces: {response:?}"
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
impl RpcProvider for HiroProvider {
    /// Proxies the request to the Stacks endpoints
    /// using the JSON-RPC schema `method` to map the endpoint and parameters.
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let json_rpc_request: JsonRpcRequest = serde_json::from_slice(&body)
            .map_err(|_| RpcError::InvalidParameter("Invalid JSON-RPC schema provided".into()))?;

        let method: SupportedMethods = serde_json::from_value(serde_json::Value::String(
            (*json_rpc_request.method).to_string(),
        ))
        .map_err(|e| RpcError::InvalidParameter(format!("Invalid method provided: {e:?}")))?;

        match method {
            SupportedMethods::StacksTransactions => {
                // Create the request body for stacks transactions endpoint schema
                // by extracting the first parameter from the JSON-RPC request and using it as
                // `tx`.
                let tx_param = json_rpc_request
                    .params
                    .as_array()
                    .and_then(|arr| arr.first())
                    .unwrap_or(&json_rpc_request.params);

                let tx = if let serde_json::Value::String(s) = tx_param {
                    s.clone()
                } else {
                    tx_param.to_string()
                };

                return self.transactions(chain_id.to_string(), tx).await;
            }
            SupportedMethods::StacksAccounts => {
                // Create the request body for stacks accounts endpoint schema
                // by extracting the first parameter from the JSON-RPC request and using it as
                // `principal`.
                let tx_param = json_rpc_request
                    .params
                    .as_array()
                    .and_then(|arr| arr.first())
                    .unwrap_or(&json_rpc_request.params);

                let tx = if let serde_json::Value::String(s) = tx_param {
                    s.clone()
                } else {
                    tx_param.to_string()
                };

                return self.accounts(chain_id.to_string(), tx).await;
            }
            SupportedMethods::HiroFeesTransaction => {
                // Create the request body for hiro fees transactions endpoint schema
                // by extracting the first parameter from the JSON-RPC request
                // and using it as `transaction_payload`.
                let tx_param = json_rpc_request
                    .params
                    .as_array()
                    .and_then(|arr| arr.first())
                    .unwrap_or(&json_rpc_request.params);

                let transaction_payload = if let serde_json::Value::String(s) = tx_param {
                    s.clone()
                } else {
                    tx_param.to_string()
                };

                return self
                    .fees_transaction(chain_id.to_string(), transaction_payload)
                    .await;
            }
            SupportedMethods::StacksTransferFees => {
                return self.transfer_fees(chain_id.to_string()).await;
            }
            SupportedMethods::StacksExtendedNonces => {
                // Create the request body for stacks extended nonces endpoint schema
                // by extracting the first parameter from the JSON-RPC request and using it as
                // `principal`.
                let tx_param = json_rpc_request
                    .params
                    .as_array()
                    .and_then(|arr| arr.first())
                    .unwrap_or(&json_rpc_request.params);

                let tx = if let serde_json::Value::String(s) = tx_param {
                    s.clone()
                } else {
                    tx_param.to_string()
                };

                return self.extended_nonces(chain_id.to_string(), tx).await;
            }
        }
    }
}

impl RpcProviderFactory<HiroConfig> for HiroProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &HiroConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        HiroProvider {
            client: forward_proxy_client,
            supported_chains,
        }
    }
}
