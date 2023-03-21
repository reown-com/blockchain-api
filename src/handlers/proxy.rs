use std::borrow::Borrow;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::analytics::MessageInfo;
use crate::error::RpcError;
use tracing::warn;

use crate::handlers::{handshake_error, RpcQueryParams};
use crate::state::State;

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
                return Ok(handshake_error("projectId", format!("{access_err:?}")));
            }
        }

        Err(err) => {
            state.metrics.add_rejected_project();
            return Ok(handshake_error("projectId", format!("{err:?}")));
        }
    }

    let chain_id = query_params.chain_id.to_lowercase();
    let provider = state.providers.get_provider_for_chain_id(&chain_id);
    let provider = match provider {
        Some(provider) => provider,
        _ => {
            return Ok(field_validation_error(
                "chainId",
                format!("We don't support the chainId you provided: {chain_id}"),
            ));
        }
    };

    state.metrics.add_rpc_call(&chain_id);

    if let Ok(rpc_request) = serde_json::from_slice(&body) {
        let (country, continent, region) = sender
            .and_then(|addr| state.analytics.lookup_geo_data(addr.ip()))
            .map(|geo| (geo.country, geo.continent, geo.region))
            .unwrap_or((None, None, None));

        state.analytics.message(MessageInfo::new(
            &query_params,
            &rpc_request,
            region,
            country,
            continent,
        ))
    }

    let project_id = query_params.project_id.clone();

    // Start timing external provider added time
    let external_call_start = SystemTime::now();

    // TODO: map the response error codes properly
    // e.g. HTTP401 from target should map to HTTP500
    let response = provider
        .proxy(method, path, query_params, headers, body)
        .await
        .map_err(|error| {
            warn!(%error, "request failed");
            if let RpcError::Throttled = error {
                state
                    .metrics
                    .add_rate_limited_call(provider.borrow(), project_id)
            }
            warp::reject::reject()
        });

    state.metrics.add_external_http_latency(
        provider.provider_kind(),
        external_call_start
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f64(),
    );

    response
}
