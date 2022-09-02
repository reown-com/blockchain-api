use crate::handlers::{new_error_response, ErrorReason, RPCQueryParams};
use crate::State;
use hyper::StatusCode;
use std::sync::Arc;
use warp::Reply;

use crate::providers::ProviderRepository;

pub async fn handler(
    state: Arc<State>,
    provider_repo: ProviderRepository,
    method: hyper::http::Method,
    path: warp::path::FullPath,
    query_params: RPCQueryParams,
    headers: hyper::http::HeaderMap,
    body: hyper::body::Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
    if query_params.project_id.is_empty() {
        return Ok(new_error_response(
            vec![ErrorReason {
                field: "projectId".to_string(),
                description: "No project id provided".to_string(),
            }],
            StatusCode::BAD_REQUEST,
        )
        .into_response());
    }

    let provider = provider_repo.get_provider("eth").unwrap();

    if !provider.supports_caip_chainid(&query_params.chain_id.to_lowercase()) {
        return Ok(new_error_response(
            vec![ErrorReason {
                field: "chainId".to_string(),
                description: "We don't support the chainId you provided".to_string(),
            }],
            StatusCode::BAD_REQUEST,
        )
        .into_response());
    }

    if let Some(metrics) = &state.metrics {
        metrics
            .rpc_call_counter
            .add(1, &[opentelemetry::KeyValue::new("chain.id", query_params.chain_id.to_lowercase())]);
    }

    // TODO: map the response error codes properly
    // e.g. HTTP401 from target should map to HTTP500
    let resp = provider
        .proxy(method, path, query_params, headers, body)
        .await;

    resp.map_err(|_e| warp::reject::reject())
}
