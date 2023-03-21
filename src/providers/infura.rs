use {
    super::{ProviderKind, RpcProvider, RpcQueryParams},
    crate::error::{RpcError, RpcResult},
    async_trait::async_trait,
    hyper::{client::HttpConnector, http, Body, Client, Response},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
};

#[derive(Clone)]
pub struct InfuraProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub project_id: String,
    pub supported_chains: HashMap<String, String>,
}

#[async_trait]
impl RpcProvider for InfuraProvider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        _path: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        _headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response<Body>> {
        let chain = self
            .supported_chains
            .get(&query_params.chain_id.to_lowercase())
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://{}.infura.io/v3/{}", chain, self.project_id);

        let hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        // TODO: map the response error codes properly
        // e.g. HTTP401 from target should map to HTTP500

        let response = self.client.request(hyper_request).await?;

        if is_rate_limited(&response) {
            return Err(RpcError::Throttled);
        }

        Ok(response)
    }

    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chainids(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Infura
    }

    fn project_id(&self) -> &str {
        &self.project_id
    }
}

fn is_rate_limited(response: &Response<Body>) -> bool {
    response.status() == http::StatusCode::TOO_MANY_REQUESTS
}
