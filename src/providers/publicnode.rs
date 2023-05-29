use {
    super::{Provider, ProviderKind, RpcProvider, RpcProviderFactory, RpcQueryParams},
    crate::{
        env::PublicnodeConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::response::{IntoResponse, Response},
    hyper::{client::HttpConnector, http, Client},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct PublicnodeProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for PublicnodeProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        // self.supported_chains
        //     .iter()
        //     .map(|(k, v)| SupportedChain {
        //         chain_id: k.clone(),
        //         weight: v.1.clone(),
        //     })
        //     .collect()

        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Publicnode
    }
}

#[async_trait]
impl RpcProvider for PublicnodeProvider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        _path: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        _headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(&query_params.chain_id.to_lowercase())
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://{}.publicnode.com", chain);

        let hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?.into_response();

        if is_rate_limited(&response) {
            return Err(RpcError::Throttled);
        }

        Ok(response)
    }
}

impl RpcProviderFactory<PublicnodeConfig> for PublicnodeProvider {
    fn new(provider_config: &PublicnodeConfig) -> Self {
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
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

fn is_rate_limited(response: &Response) -> bool {
    response.status() == http::StatusCode::TOO_MANY_REQUESTS
}
