use crate::State;
use hyper::{client::HttpConnector, Client};
use std::sync::Arc;

pub async fn handler(
    _state: Arc<State>,
    client: Client<HttpConnector>,
    method: hyper::http::Method,
    path: warp::path::FullPath,
    query_params: String,
    headers: hyper::http::HeaderMap,
    body: hyper::body::Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO: do some validation
    let mut req = {
        let mut full_path = path.as_str().to_string();
        if query_params != "" {
            full_path = format!("{}?{}", full_path, query_params);
        }
        let mut hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(full_path)
            .body(hyper::body::Body::from(body))
            .expect("Request::builder() failed");
        {
            *hyper_request.headers_mut() = headers;
        }
        hyper_request
    };

    // TODO: use RPC provider strategy
    *req.uri_mut() = "http://httpbin.org/ip"
        .parse()
        .expect("Failed to parse the uri");

    // TODO: map the response error codes properly
    // e.g. HTTP401 from target should map to HTTP500
    client
        .request(req)
        .await
        .map_err(|_e| warp::reject::reject())
}
