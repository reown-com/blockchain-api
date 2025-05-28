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
    serde::Serialize,
    std::collections::HashMap,
    tracing::debug,
};

#[derive(Debug)]
pub struct HiroProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub supported_chains: HashMap<String, String>,
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

#[async_trait]
impl RpcProvider for HiroProvider {
    /// Proxies the request to the Stacks `/v2/transactions` endpoint only
    /// using the JSON-RPC schema with the `stacks_transactions` method
    /// and a single parameter as `tx`.
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;
        let uri = format!("{uri}/v2/transactions");
        let uri = uri.parse::<hyper::Uri>().map_err(|_| {
            RpcError::InvalidParameter("Failed to parse URI for stacks_transactions".into())
        })?;

        let json_rpc_request: JsonRpcRequest = serde_json::from_slice(&body)
            .map_err(|_| RpcError::InvalidParameter("Invalid JSON-RPC schema provided".into()))?;

        if json_rpc_request.method != "stacks_transactions".into() {
            return Err(RpcError::InvalidParameter(
                "Invalid method provided. Only 'stacks_transactions' is currently supported".into(),
            ));
        }

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
                 Hiro: {response:?}"
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
