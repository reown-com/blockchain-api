use super::{ProviderKind, RpcProvider, RpcQueryParams};
use crate::error::{RpcError, RpcResult};
use async_trait::async_trait;
use hyper::{client::HttpConnector, http, Body, Client, Response};
use hyper_tls::HttpsConnector;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PoktProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub project_id: String,
    pub supported_chains: HashMap<String, String>,
}

#[async_trait]
impl RpcProvider for PoktProvider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        _path: warp::path::FullPath,
        query_params: RpcQueryParams,
        _headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response<Body>> {
        let chain = self
            .supported_chains
            .get(&query_params.chain_id.to_lowercase())
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!(
            "https://{}.gateway.pokt.network/v1/lb/{}",
            chain, self.project_id
        );

        let hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        // TODO: map the response error codes properly
        // e.g. HTTP401 from target should map to HTTP500
        Ok(self.client.request(hyper_request).await?)
    }

    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chainids(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Pokt
    }

    fn project_id(&self) -> String {
        self.project_id.clone()
    }

    fn is_rate_limited(&self, response: &Response<Body>) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}
