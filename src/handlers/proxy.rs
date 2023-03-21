use {
    crate::{
        analytics::MessageInfo,
        error::{self, RpcError},
        extractors::method::Method,
        handlers::RpcQueryParams,
        json_rpc::JsonRpcRequest,
        state::AppState,
    },
    axum::{
        body::Bytes,
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::{HeaderMap, StatusCode},
    std::{
        borrow::Borrow,
        net::SocketAddr,
        sync::Arc,
        time::{Duration, SystemTime},
    },
    tracing::{info, warn},
};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query_params): Query<RpcQueryParams>,
    Method(method): Method,
    path: MatchedPath,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RpcError> {
    let project = state
        .registry
        .project_data(&query_params.project_id)
        .await?;

    project.validate_access(&query_params.project_id, None)?;

    let chain_id = query_params.chain_id.to_lowercase();
    let provider = state
        .providers
        .get_provider_for_chain_id(&chain_id)
        .ok_or(RpcError::UnsupportedChain(chain_id.clone()))?;

    state.metrics.add_rpc_call(&chain_id);

    if let Ok(rpc_request) = serde_json::from_slice(&body) {
        let (country, continent, region) = state
            .analytics
            .lookup_geo_data(addr.ip())
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
            return RpcError::ProviderError;
        })?;

    state.metrics.add_external_http_latency(
        provider.provider_kind(),
        external_call_start
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f64(),
    );

    Ok(response.into_response())
}
