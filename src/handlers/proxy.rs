use {
    super::HANDLER_TASK_METRICS,
    crate::{analytics::MessageInfo, error::RpcError, handlers::RpcQueryParams, state::AppState},
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
    tracing::{
        info,
        log::{error, warn},
    },
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    addr: ConnectInfo<SocketAddr>,
    query_params: Query<RpcQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RpcError> {
    handler_internal(state, addr, query_params, path, headers, body)
        .with_metrics(HANDLER_TASK_METRICS.with_name("proxy"))
        .await
}

async fn handler_internal(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query_params): Query<RpcQueryParams>,
    _path: MatchedPath,
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
        .tap_err(|e| {
            state.metrics.add_rejected_project();
            info!(
                "Denied access for project: {}, with reason: {}",
                query_params.project_id, e
            );
        })?;

    let chain_id = query_params.chain_id.to_lowercase();

    let provider = state
        .providers
        .get_provider_for_chain_id(&chain_id)
        .ok_or(RpcError::UnsupportedChain(chain_id.clone()))?;

    state.metrics.add_rpc_call(chain_id.clone());

    let origin = headers
        .get("origin")
        .map(|v| v.to_str().unwrap_or("invalid_header").to_string());

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
            &provider.provider_kind(),
            origin,
        ));
    }

    let project_id = query_params.project_id.clone();

    // Start timing external provider added time
    let external_call_start = SystemTime::now();

    let mut response = provider.proxy(&chain_id, body).await.tap_err(|e| {
        warn!(
            "Failed call to provider: {} with {}",
            provider.provider_kind(),
            e
        );
    })?;

    state
        .metrics
        .add_status_code_for_provider(provider.borrow(), response.status(), chain_id);

    if provider.is_rate_limited(&mut response).await {
        state
            .metrics
            .add_rate_limited_call(provider.borrow(), project_id);
        *response.status_mut() = http::StatusCode::BAD_GATEWAY;
    }

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
        _ => {
            error!(
                "Call to provider '{}' failed with status '{}' and body '{:?}'",
                provider.provider_kind(),
                response.status(),
                response.body()
            );
            state.metrics.add_failed_provider_call(provider.borrow());
        }
    };
    Ok(response)
}
