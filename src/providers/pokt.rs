use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::PoktConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    serde::Deserialize,
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct PoktProvider {
    pub client: reqwest::Client,
    pub supported_chains: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct InternalErrorResponse {
    pub request_id: String,
    pub error: String,
}

impl Provider for PoktProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Pokt
    }
}

#[async_trait]
impl RateLimited for PoktProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == hyper::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for PoktProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

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

impl RpcProviderFactory<PoktConfig> for PoktProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &PoktConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        PoktProvider {
            client: forward_proxy_client,
            supported_chains,
        }
    }
}
