use std::sync::Arc;

use tap::TapFallible;
use tracing::warn;

use crate::handlers::{handshake_error, RpcQueryParams};
use crate::providers::ProviderRepository;
use crate::State;

use super::field_validation_error;

pub async fn handler(
    state: Arc<State>,
    provider_repo: ProviderRepository,
    method: hyper::http::Method,
    path: warp::path::FullPath,
    query_params: RpcQueryParams,
    headers: hyper::http::HeaderMap,
    body: hyper::body::Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
    if query_params.project_id.is_empty() {
        return Ok(field_validation_error(
            "projectId",
            "No project id provided",
        ));
    }

    match state.registry.project_data(&query_params.project_id).await {
        Ok(project) => {
            if let Err(access_err) = project.validate_access(&query_params.project_id, None) {
                state.metrics.add_rejected_project();
                return Ok(handshake_error("projectId", format!("{:?}", access_err)));
            }
        }

        Err(err) => {
            state.metrics.add_rejected_project();
            return Ok(handshake_error("projectId", format!("{:?}", err)));
        }
    }

    let chain_id = query_params.chain_id.to_lowercase();
    let provider = provider_repo.get_provider_for_chain_id(&chain_id);
    let provider = match provider {
        Some(provider) => provider,
        _ => {
            return Ok(field_validation_error(
                "chainId",
                format!("We don't support the chainId you provided: {}", chain_id),
            ))
        }
    };

    state.metrics.add_rpc_call(&chain_id);

    // TODO: map the response error codes properly
    // e.g. HTTP401 from target should map to HTTP500
    provider
        .proxy(method, path, query_params, headers, body)
        .await
        .tap_err(|error| warn!(%error, "request failed"))
        .map_err(|_| warp::reject::reject())
}
