use {
    super::{RpcQueryParams, HANDLER_TASK_METRICS},
    crate::{
        analytics::MessageInfo,
        error::RpcError,
        state::AppState,
        utils::{crypto, network},
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
    tokio::time::timeout,
    tracing::{
        log::{debug, error, warn},
        Span,
    },
    wc::future::FutureExt,
};

const PROVIDER_PROXY_MAX_CALLS: usize = 3;
const PROVIDER_PROXY_CALL_TIMEOUT: Duration = Duration::from_secs(10);

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

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query_params): Query<RpcQueryParams>,
    _path: MatchedPath,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query_params.project_id.clone())
        .await?;
    rpc_call(state, addr, query_params, headers, body).await
}

#[tracing::instrument(skip(state), level = "debug")]
pub async fn rpc_call(
    state: Arc<AppState>,
    addr: SocketAddr,
    query_params: RpcQueryParams,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RpcError> {
    let chain_id = query_params.chain_id.clone();
    // Exact provider proxy request for testing suite
    // This request is allowed only for the RPC_PROXY_TESTING_PROJECT_ID
    let providers = match query_params.provider_id.clone() {
        Some(provider_id) => {
            let provider = vec![state
                .providers
                .get_provider_by_provider_id(&provider_id)
                .ok_or_else(|| RpcError::UnsupportedProvider(provider_id.clone()))?];

            if let Some(ref testing_project_id) = state.config.server.testing_project_id {
                if !crypto::constant_time_eq(testing_project_id, &query_params.project_id) {
                    return Err(RpcError::InvalidParameter(format!(
                        "The project ID {} is not allowed to use the exact provider request",
                        query_params.project_id
                    )));
                }
            } else {
                return Err(RpcError::InvalidParameter(
                    "RPC_PROXY_TESTING_PROJECT_ID should be configured for this type of request"
                        .into(),
                ));
            }

            provider
        }
        None => state
            .providers
            .get_provider_for_chain_id(&chain_id, PROVIDER_PROXY_MAX_CALLS)?,
    };

    for (i, provider) in providers.iter().enumerate() {
        let response = rpc_provider_call(
            state.clone(),
            addr,
            query_params.clone(),
            headers.clone(),
            body.clone(),
            provider.clone(),
        )
        .await;

        match response {
            Ok(response) if !response.status().is_server_error() => {
                return Ok(response);
            }
            e => {
                state
                    .metrics
                    .add_rpc_call_retries(i as u64, chain_id.clone());
                debug!(
                    "Provider '{}' returned an error {e:?}, trying the next provider",
                    provider.provider_kind()
                );
            }
        }
    }

    state.metrics.add_no_providers_for_chain(chain_id.clone());
    debug!("All providers failed for chain_id: {}", chain_id);
    Err(RpcError::ChainTemporarilyUnavailable(chain_id))
}

#[tracing::instrument(skip(state), level = "debug")]
pub async fn rpc_provider_call(
    state: Arc<AppState>,
    addr: SocketAddr,
    query_params: RpcQueryParams,
    headers: HeaderMap,
    body: Bytes,
    provider: Arc<dyn crate::providers::RpcProvider>,
) -> Result<Response, RpcError> {
    Span::current().record("provider", provider.provider_kind().to_string());
    let chain_id = query_params.chain_id.clone();
    let origin = headers
        .get("origin")
        .map(|v| v.to_str().unwrap_or("invalid_header").to_string());

    state.metrics.add_rpc_call(chain_id.clone());
    if let Ok(rpc_request) = serde_json::from_slice(&body) {
        let (country, continent, region) = state
            .analytics
            .lookup_geo_data(network::get_forwarded_ip(headers).unwrap_or_else(|| addr.ip()))
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

    let proxy_fut = provider.proxy(&chain_id, body);
    let timeout_fut = timeout(PROVIDER_PROXY_CALL_TIMEOUT, proxy_fut);
    let mut response = timeout_fut
        .await
        .tap_err(|e| {
            warn!(
                "Timeout calling provider: {} with {}",
                provider.provider_kind(),
                e
            );
        })
        .map_err(RpcError::ProxyTimeoutError)?
        .tap_err(|e| {
            warn!(
                "Failed call to provider: {} with {}",
                provider.provider_kind(),
                e
            );
        })?;

    state.metrics.add_status_code_for_provider(
        provider.provider_kind(),
        response.status().as_u16(),
        Some(chain_id),
        None,
    );

    if provider.is_rate_limited(&mut response).await {
        state
            .metrics
            .add_rate_limited_call(provider.borrow(), project_id);
        *response.status_mut() = http::StatusCode::SERVICE_UNAVAILABLE;
    }

    state
        .metrics
        .add_external_http_latency(provider.provider_kind(), external_call_start, None);

    match response.status() {
        http::StatusCode::OK | http::StatusCode::BAD_REQUEST => {
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
            *response.status_mut() = http::StatusCode::SERVICE_UNAVAILABLE;
        }
    };
    Ok(response)
}
