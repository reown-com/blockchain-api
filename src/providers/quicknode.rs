use {
    super::{
        is_internal_error_rpc_code, is_node_error_rpc_message, is_rate_limited_error_rpc_message,
        Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory,
    },
    crate::{
        env::QuicknodeConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::{client::HttpConnector, http, Client, Method},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
    tracing::debug,
};

#[derive(Debug)]
pub struct QuicknodeProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
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

#[async_trait]
impl RateLimited for QuicknodeProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool
    where
        Self: Sized,
    {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for QuicknodeProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
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
        let uri = format!("https://{chain_subdomain}.quiknode.pro/{token}");

        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if let Some(error) = &response.error {
                if status.is_success() {
                    debug!(
                        "Strange: provider returned JSON RPC error, but status {status} is success: \
                     Aurora: {response:?}"
                    );
                    // Handle internal error codes with message-based classification
                    if is_internal_error_rpc_code(error.code) {
                        if is_rate_limited_error_rpc_message(&error.message) {
                            return Ok((http::StatusCode::TOO_MANY_REQUESTS, body).into_response());
                        }
                        if is_node_error_rpc_message(&error.message) {
                            return Ok(
                                (http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
                            );
                        }
                    }
                }
            }
        }

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
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
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
