use crate::handlers::{new_error_response, ErrorReason};
use crate::State;
use hyper::StatusCode;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use std::sync::Arc;
use warp::Reply;

#[derive(Deserialize)]
pub struct RPCQueryParams {
    #[serde(rename = "chainId")]
    chain_id: String,
    #[serde(rename = "projectId")]
    project_id: String,
}

pub async fn handler(
    state: Arc<State>,
    client: Client<HttpsConnector<HttpConnector>>,
    method: hyper::http::Method,
    path: warp::path::FullPath,
    query_params: RPCQueryParams,
    headers: hyper::http::HeaderMap,
    body: hyper::body::Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
    if query_params.chain_id != "eth:1" {
        return Ok(new_error_response(
            vec![ErrorReason {
                field: "chainId".to_string(),
                description: "We currently only support `eth:1`".to_string(),
            }],
            StatusCode::BAD_REQUEST,
        )
        .into_response());
    }

    if query_params.project_id.is_empty() {
        return Ok(new_error_response(
            vec![ErrorReason {
                field: "projectId".to_string(),
                description: "No project id required".to_string(),
            }],
            StatusCode::BAD_REQUEST,
        )
        .into_response());
    }

    let mut req = {
        let full_path = path.as_str().to_string();
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
    *req.uri_mut() = format!(
        "https://mainnet.infura.io/v3/{}",
        state.config.infura_project_id
    )
    .parse()
    .expect("Failed to parse the uri");

    // TODO: map the response error codes properly
    // e.g. HTTP401 from target should map to HTTP500
    // TODO: stream not `await` this
    let resp = client.request(req).await;
    resp.map_err(|_e| warp::reject::reject())
}
