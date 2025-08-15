use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::BinanceConfig,
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
pub struct BinanceProvider {
    pub client: HyperClientLegacy<hyper_rustls::HttpsConnector<HttpConnector>, axum::body::Body>,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for BinanceProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Binance
    }
}

#[async_trait]
impl RateLimited for BinanceProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool
    where
        Self: Sized,
    {
        response.status() == http::StatusCode::FORBIDDEN
    }
}

#[async_trait]
impl RpcProvider for BinanceProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

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

impl RpcProviderFactory<BinanceConfig> for BinanceProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &BinanceConfig) -> Self {
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

        BinanceProvider {
            client: forward_proxy_client,
            supported_chains,
        }
    }
}
