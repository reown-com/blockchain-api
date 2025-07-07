use {
    super::{
        is_internal_error_rpc_code, is_node_error_rpc_message, is_rate_limited_error_rpc_message,
        Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory,
    },
    crate::{
        env::BlastConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::{self, client::HttpConnector, http, Client, Method, StatusCode},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
    tracing::debug,
};

#[derive(Debug)]
pub struct BlastProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub api_key: String,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for BlastProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Blast
    }
}

#[async_trait]
impl RateLimited for BlastProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for BlastProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://{}.blastapi.io/{}", chain, self.api_key);

        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await?;

        if let Ok(json_response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if let Some(error) = &json_response.error {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                 Blast: {json_response:?}"
                );
                if is_internal_error_rpc_code(error.code) {
                    if is_rate_limited_error_rpc_message(&error.message) {
                        return Ok((http::StatusCode::TOO_MANY_REQUESTS, body).into_response());
                    }
                    if is_node_error_rpc_message(&error.message) {
                        return Ok((http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response());
                    }
                }
            }
        }

        let response = (
            status,
            [(
                hyper::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )],
            body,
        )
            .into_response();
        Ok(response)
    }
}

impl RpcProviderFactory<BlastConfig> for BlastProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &BlastConfig) -> Self {
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        BlastProvider {
            client: forward_proxy_client,
            supported_chains,
            api_key: provider_config.api_key.clone(),
        }
    }
}
