use super::{RPCProvider, RPCQueryParams};
use async_trait::async_trait;
use hyper::Error;
use hyper::{client::HttpConnector, Body, Client, Response};
use hyper_tls::HttpsConnector;

#[derive(Clone)]
pub struct InfuraProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub infura_project_id: String,
}

#[async_trait]
impl RPCProvider for InfuraProvider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        path: warp::path::FullPath,
        _query_params: RPCQueryParams,
        _headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> Result<Response<Body>, Error> {
        let mut req = {
            let full_path = path.as_str().to_string();
            let hyper_request = hyper::http::Request::builder()
                .method(method)
                .uri(full_path)
                .header("Content-Type", "application/json")
                .body(hyper::body::Body::from(body))
                .expect("Request::builder() failed");
            hyper_request
        };

        *req.uri_mut() = format!("https://mainnet.infura.io/v3/{}", self.infura_project_id)
            .parse()
            .expect("Failed to parse the uri");

        // TODO: map the response error codes properly
        // e.g. HTTP401 from target should map to HTTP500
        self.client.request(req).await
    }
}
