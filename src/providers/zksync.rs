use {
    super::{Provider, ProviderKind, RpcProvider, RpcProviderFactory, RpcQueryParams},
    crate::{
        env::ZKSyncConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::response::{IntoResponse, Response},
    hyper::{client::HttpConnector, http, Body, Client},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct ZKSyncProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for ZKSyncProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::ZKSync
    }
}

#[async_trait]
impl RpcProvider for ZKSyncProvider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        _path: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        _headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&query_params.chain_id.to_lowercase())
            .ok_or(RpcError::ChainNotFound)?;

        let hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;

        if is_rate_limited(&response) {
            return Err(RpcError::Throttled);
        }

        Ok(response.into_response())
    }
}

impl RpcProviderFactory<ZKSyncConfig> for ZKSyncProvider {
    fn new(provider_config: &ZKSyncConfig) -> Self {
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        ZKSyncProvider {
            client: forward_proxy_client,
            supported_chains,
        }
    }
}

fn is_rate_limited(response: &Response<Body>) -> bool {
    response.status() == http::StatusCode::TOO_MANY_REQUESTS
}
