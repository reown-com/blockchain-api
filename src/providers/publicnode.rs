use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::PublicnodeConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    http_body_util::BodyExt,
    hyper::{http, Method},
    hyper_rustls::HttpsConnectorBuilder,
    hyper_util::client::legacy::{connect::HttpConnector, Client as HyperClientLegacy},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct PublicnodeProvider {
    pub client: HyperClientLegacy<hyper_rustls::HttpsConnector<HttpConnector>, axum::body::Body>,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for PublicnodeProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Publicnode
    }
}

#[async_trait]
impl RateLimited for PublicnodeProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for PublicnodeProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://{chain}.publicnode.com");

        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;
        let status = response.status();
        let body = response.into_body().collect().await?.to_bytes();
        let mut response = (status, body).into_response();
        response
            .headers_mut()
            .insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok(response)
    }
}

impl RpcProviderFactory<PublicnodeConfig> for PublicnodeProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &PublicnodeConfig) -> Self {
        let https = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_only()
            .enable_http1()
            .build();
        let forward_proxy_client: HyperClientLegacy<_, axum::body::Body> =
            HyperClientLegacy::builder(hyper_util::rt::TokioExecutor::new()).build(https);
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        PublicnodeProvider {
            client: forward_proxy_client,
            supported_chains,
        }
    }
}
