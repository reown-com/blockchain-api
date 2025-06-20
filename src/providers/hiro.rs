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
    hyper::{client::HttpConnector, http, Client, Method},
    hyper_tls::HttpsConnector,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    tracing::debug,
};

#[derive(Debug)]
pub struct HiroProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub supported_chains: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportedMethods {
    StacksTransactions,
    StacksAccounts,
}

#[derive(Debug, Serialize)]
pub struct TransactionsRequest {
    pub tx: String,
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
    // Send request to the Stacks `/v2/transactions` endpoint
    async fn transactions(&self, chain_id: String, tx: String) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&chain_id)
            .ok_or(RpcError::ChainNotFound)?;
        let uri = format!("{}/v2/transactions", uri.trim_end_matches('/'));
        let uri = uri.parse::<hyper::Uri>().map_err(|_| {
            RpcError::InvalidParameter("Failed to parse URI for stacks_transactions".into())
        })?;

        let stacks_transactions_request = serde_json::to_string(&TransactionsRequest { tx })
            .map_err(|e| {
                RpcError::InvalidParameter(format!("Failed to serialize transaction: {}", e))
            })?;

        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(stacks_transactions_request))?;

        let response = self.client.request(hyper_request).await?;
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                 Hiro transactions: {response:?}"
                );
            }
        }

        let mut response = (status, body).into_response();
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
        let uri = uri.parse::<hyper::Uri>().map_err(|_| {
            RpcError::InvalidParameter("Failed to parse URI for stacks_accounts".into())
        })?;

        let hyper_request = hyper::http::Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::empty())?;

        let response = self.client.request(hyper_request).await?;
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                 Hiro accounts: {response:?}"
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
        .map_err(|e| RpcError::InvalidParameter(format!("Invalid method provided: {:?}", e)))?;

        match method {
            SupportedMethods::StacksTransactions => {
                // Create the request body for stacks transactions endpoint schema
                // by extracting the first parameter from the JSON-RPC request and using it as `tx`.
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
                // by extracting the first parameter from the JSON-RPC request and using it as `principal`.
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
        }
    }
}

impl RpcProviderFactory<HiroConfig> for HiroProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &HiroConfig) -> Self {
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
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
