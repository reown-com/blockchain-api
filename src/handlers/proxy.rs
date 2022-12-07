use std::borrow::Borrow;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::analytics::MessageInfo;
use hyper::body::Bytes;
use hyper::{body, Body, Response};
use tracing::warn;

use crate::handlers::{handshake_error, RpcQueryParams};
use crate::State;

use super::field_validation_error;

pub async fn handler(
    state: Arc<State>,
    sender: Option<SocketAddr>,
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
    let provider = state.providers.get_provider_for_chain_id(&chain_id);
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

    if let Ok(rpc_request) = serde_json::from_slice(&body) {
        let (country, continent) = sender
            .and_then(|addr| state.analytics.lookup_geo_data(addr.ip()))
            .map(|geo| (geo.country, geo.continent))
            .unwrap_or((None, None));
        state.analytics.message(MessageInfo::new(
            &query_params,
            &rpc_request,
            sender,
            country,
            continent,
        ))
    }

    let project_id = query_params.project_id.clone();

    // TODO: map the response error codes properly
    // e.g. HTTP401 from target should map to HTTP500
    let response = provider
        .proxy(method, path, query_params, headers, body)
        .await;

    match response {
        Err(err) => {
            warn!(%err, "request failed");
            Err(warp::reject::reject())
        }
        Ok(response) => match copy_body_bytes(response).await {
            Ok((body_bytes, response)) => {
                if provider.is_rate_limited(&response, body_bytes) {
                    state
                        .metrics
                        .add_rate_limited_call(provider.borrow(), project_id)
                };
                Ok(response)
            }
            Err(err) => {
                warn!(%err, "failed converting body to bytes");
                Err(warp::reject::reject())
            }
        },
    }
}

async fn copy_body_bytes(
    response: Response<Body>,
) -> Result<(Bytes, Response<Body>), hyper::Error> {
    let (parts, body) = response.into_parts();
    let bytes = body::to_bytes(body).await?;

    let body = Body::from(bytes.clone());
    Ok((bytes, Response::from_parts(parts, body)))
}
