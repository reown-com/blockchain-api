use {
    crate::{
        analytics::MessageInfo,
        error::RpcError,
        extractors::method::Method,
        handlers::RpcQueryParams,
        state::AppState,
    },
    axum::{
        body::Bytes,
        extract::{ConnectInfo, MatchedPath, Query, State},
        response::Response,
    },
    hyper::{http, HeaderMap},
    std::{
        borrow::Borrow,
        net::SocketAddr,
        sync::Arc,
        time::{Duration, SystemTime},
    },
    tap::TapFallible,
    tracing::warn,
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
        .await
        .tap_err(|_| state.metrics.add_rejected_project())?;

    project
        .validate_access(&query_params.project_id, None)
        .tap_err(|_| state.metrics.add_rejected_project())?;

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
            provider.provider_kind(),
        ))
    }

    let project_id = query_params.project_id.clone();

    // Start timing external provider added time
    let external_call_start = SystemTime::now();

    let mut response = provider
        .proxy(method, path, query_params, headers, body)
        .await
        .map_err(|error| {
            warn!(%error, "request failed");
            if let RpcError::Throttled = error {
                state
                    .metrics
                    .add_rate_limited_call(provider.borrow(), project_id)
            }
            RpcError::ProviderError
        })?;

    state.metrics.add_external_http_latency(
        provider.provider_kind(),
        external_call_start
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f64(),
    );

    match response.status() {
        http::StatusCode::OK => {
            state.metrics.add_finished_provider_call(provider.borrow());
        }
        status => {
            state.metrics.add_failed_provider_call(provider.borrow());
            state
                .metrics
                .add_status_code_for_provider(provider.borrow(), status);

            *response.status_mut() = http::StatusCode::BAD_GATEWAY;
        }
    };

    Ok(response)
}
