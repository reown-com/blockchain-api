use super::{RPCProvider, RPCQueryParams};
use async_trait::async_trait;
use hyper::Error;
use hyper::{client::HttpConnector, Body, Client, Response};
use hyper_tls::HttpsConnector;
use std::collections::HashMap;

#[derive(Clone)]
pub struct InfuraProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub infura_project_id: String,
    pub infura_supported_chains: HashMap<String, String>,
}

#[async_trait]
impl RPCProvider for InfuraProvider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        path: warp::path::FullPath,
        query_params: RPCQueryParams,
        _headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> Result<Response<Body>, Error> {
        let full_path = path.as_str().to_string();
        let mut hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(full_path)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))
            .expect("Request::builder() failed");

        let chain = self
            .infura_supported_chains
            .get(&query_params.chain_id.to_lowercase())
            .expect("Chain not found despite previous validation");

        *hyper_request.uri_mut() =
            format!("https://{}.infura.io/v3/{}", chain, self.infura_project_id)
                .parse()
                .expect("Failed to parse the uri");

        // TODO: map the response error codes properly
        // e.g. HTTP401 from target should map to HTTP500
        self.client.request(hyper_request).await
    }

    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.infura_supported_chains.contains_key(chain_id)
    }
}
