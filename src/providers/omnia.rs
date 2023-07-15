use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::OmniatechConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::response::{IntoResponse, Response},
    hyper::{client::HttpConnector, Client, Method, StatusCode},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct OmniatechProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for OmniatechProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Omniatech
    }
}

#[async_trait]
impl RateLimited for OmniatechProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool
    where
        Self: Sized,
    {
        response.status() == StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for OmniatechProvider {
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let chain = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://endpoints.omniatech.io/v1/{}/mainnet/public", chain);

        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?.into_response();

        Ok(response)
    }
}

impl RpcProviderFactory<OmniatechConfig> for OmniatechProvider {
    fn new(provider_config: &OmniatechConfig) -> Self {
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        OmniatechProvider {
            client: forward_proxy_client,
            supported_chains,
        }
    }
}
