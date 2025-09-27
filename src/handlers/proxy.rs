use {
    super::RpcQueryParams,
    crate::{
        analytics::MessageInfo,
        error::RpcError,
        json_rpc::JsonRpcRequest,
        providers::{
            is_internal_error_rpc_code, is_known_rpc_error_message, is_node_error_rpc_message,
            is_rate_limited_error_rpc_message, ProviderKind,
        },
        state::AppState,
        utils::{
            batch_json_rpc_request::MaybeBatchRequest, crypto, json_rpc_cache::is_cached_response,
            network,
        },
    },
    axum::{
        body::{to_bytes, Bytes},
        extract::{ConnectInfo, Query, State},
        response::{IntoResponse, Response},
    },
    hyper::{http, HeaderMap},
    std::{
        borrow::Borrow,
        collections::HashSet,
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
    wc::metrics::{future_metrics, FutureExt},
};

const PROVIDER_PROXY_MAX_CALLS: usize = 5;
const PROVIDER_PROXY_CALL_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_CONTENT_TYPE: (&str, &str) = ("content-type", "application/json");
pub const PROVIDER_RESPONSE_MAX_BYTES: usize = 10 * 1024 * 1024; // 10 Mb

pub async fn handler(
    state: State<Arc<AppState>>,
    addr: ConnectInfo<SocketAddr>,
    query_params: Query<RpcQueryParams>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RpcError> {
    handler_internal(state, addr, query_params, headers, body)
        .with_metrics(future_metrics!("handler_task", "name" => "proxy"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query_params): Query<RpcQueryParams>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RpcError> {
    // Don't validate the quota and validate project access only
    // if the chainId is in the skip_quota_chains list
    if state
        .config
        .server
        .skip_quota_chains
        .contains(&query_params.chain_id)
    {
        state
            .validate_project_access(&query_params.project_id.clone())
            .await?;
    } else {
        state
            .validate_project_access_and_quota(&query_params.project_id.clone())
            .await?;
    };

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

    // Deserializing the request body to a JSON-RPC request schema and
    // check if a cached response can be returned
    // TODO: Optimize this to remove the second deserialization during the provider analytics
    match serde_json::from_slice::<JsonRpcRequest>(&body) {
        Ok(request) => {
            if let Some(response) =
                is_cached_response(&chain_id, &request, &state.metrics, &state.moka_cache).await
            {
                return Ok((
                    http::StatusCode::OK,
                    [DEFAULT_CONTENT_TYPE],
                    serde_json::to_string(&response)?,
                )
                    .into_response());
            }
        }
        Err(e) => {
            error!("Failed to deserialize JSON-RPC request: {e}");
        }
    };

    if query_params.session_id.is_some() {
        let provider_kind = match chain_id.as_str() {
            "eip155:10" => Some(ProviderKind::Quicknode), // Optimism
            "eip155:8453" => Some(ProviderKind::Publicnode), // Base
            "eip155:42161" => Some(ProviderKind::Publicnode), // Arbitrum One
            _ => {
                debug!(
                    "Requested sessionId for chain {chain_id} but no hardcoded provider was configured"
                );
                None
            }
        };

        if let Some(provider_kind) = provider_kind {
            let provider = state
                .providers
                .get_rpc_provider_by_provider_kind(&provider_kind)
                .ok_or_else(|| RpcError::UnsupportedProvider(provider_kind.to_string()))?;
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
                    // No metrics are recorded for these hardcoded providers since it bypasses our routign algorithm
                    return Ok(response);
                }
                e => {
                    // Not recording metric since this is a hardcoded provider
                    // state
                    //     .metrics
                    //     .add_rpc_call_retries(0, chain_id.clone());
                    debug!(
                        "Provider (via sessionId) '{}' returned an error {e:?}, trying the next provider",
                        provider.provider_kind()
                    );
                }
            }
        }
    }

    // Start timing the total chain request (including retries)
    let chain_request_start = SystemTime::now();

    // Exact provider proxy request for testing suite
    // This request is allowed only for the RPC_PROXY_TESTING_PROJECT_ID
    let providers = match query_params.provider_id.clone() {
        Some(provider_id) => {
            let provider = vec![state
                .providers
                .get_rpc_provider_by_provider_id(&provider_id)
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
            .get_rpc_provider_for_chain_id(&chain_id, PROVIDER_PROXY_MAX_CALLS)?,
    };

    for (i, provider) in providers.iter().enumerate() {
        let provider_call = rpc_provider_call(
            state.clone(),
            addr,
            query_params.clone(),
            headers.clone(),
            body.clone(),
            provider.clone(),
        )
        .await;

        let response_result = match provider_call {
            Ok(response) => response,
            Err(e) => {
                error!(
                    "Call to provider '{}' returned a connection error {e:?}, trying the next provider",
                    provider.provider_kind()
                );
                state
                    .metrics
                    .add_provider_connection_error(chain_id.clone(), provider.borrow());
                state
                    .metrics
                    .add_rpc_call_retries(i as u64, chain_id.clone());
                continue;
            }
        };

        // Proceed to the result if the status is success or
        // any client error except the rate limited error
        let provider_kind = provider.provider_kind();
        let status = response_result.status();
        if status.is_success() || status == http::StatusCode::BAD_REQUEST {
            let body_bytes =
                match to_bytes(response_result.into_body(), PROVIDER_RESPONSE_MAX_BYTES).await {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!(
                        "Failed to read JSON-RPC response body from provider {provider_kind}: {e}"
                    );
                        state
                            .metrics
                            .add_rpc_call_retries(i as u64, chain_id.clone());
                        continue;
                    }
                };

            // Check the JSON-RPC response schema and possible internal error codes
            match serde_json::from_slice::<jsonrpc::Response>(&body_bytes) {
                Ok(json_response) => {
                    if let Some(error) = &json_response.error {
                        let error_code = error.code;
                        let error_message = error.message.clone();

                        // Internal error codes range -32000..-32099 https://www.jsonrpc.org/specification#error_object
                        if is_internal_error_rpc_code(error_code) {
                            // Retry to another provider if the error is a rate limited or node error
                            if is_rate_limited_error_rpc_message(&error_message)
                                || is_node_error_rpc_message(&error_message)
                            {
                                state
                                    .metrics
                                    .add_rpc_call_retries(i as u64, chain_id.clone());
                                continue;
                            }

                            // Log an error, increment the metrics for unknown error codes and continue
                            // without retrying since it can be a contract execution error.
                            // We should catch unknown errors by alarm for the metrics
                            // and investigate it first without retrying.
                            if !is_known_rpc_error_message(&error_message) {
                                error!("Provider {provider_kind} returned an error code: {error_code} and the message: {error_message}");
                                state.metrics.add_internal_error_code_for_provider(
                                    provider_kind,
                                    chain_id.clone(),
                                    error.code,
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse JSON-RPC response from provider {provider_kind}: {e}. Message: {}", String::from_utf8_lossy(&body_bytes));
                }
            }

            state
                .metrics
                .add_found_provider_for_chain(chain_id.clone(), &provider.provider_kind());

            // Record successful chain latency for the provider that succeeded
            // and return the response
            state.metrics.add_chain_latency(
                &provider.provider_kind(),
                chain_request_start,
                chain_id.clone(),
            );
            return Ok((status, [DEFAULT_CONTENT_TYPE], body_bytes).into_response());
        }

        debug!(
            "Provider '{}' returned unsuccessful status {}, trying the next provider",
            provider.provider_kind(),
            status
        );
        state
            .metrics
            .add_rpc_call_retries(i as u64, chain_id.clone());
    }

    state.metrics.add_no_providers_for_chain(chain_id.clone());
    debug!("All providers failed for chain_id: {chain_id}");
    Err(RpcError::ChainTemporarilyUnavailable(chain_id))
}

// TODO eventually refactor this to be called by the wallet handler (generic JSON-RPC)
// However, dependency on us having an exhaustive list of supported RPC methods is a blocker to merging these handlers.
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
        .map(|v| Arc::from(v.to_str().unwrap_or("invalid_header").to_string()));

    state
        .metrics
        .add_rpc_call(chain_id.clone(), &provider.provider_kind());

    let (country, continent, region) = state
        .analytics
        .lookup_geo_data(network::get_forwarded_ip(&headers).unwrap_or_else(|| addr.ip()))
        .map(|geo| (geo.country, geo.continent, geo.region))
        .unwrap_or((None, None, None));

    match serde_json::from_slice::<MaybeBatchRequest>(&body) {
        Ok(body) => {
            let rpcs = match &body {
                MaybeBatchRequest::Single(req) => {
                    vec![(req.id.to_string(), req.method.to_string())]
                }
                MaybeBatchRequest::Batch(reqs) => {
                    {
                        // Validate unique RPC IDs
                        let mut ids = HashSet::new();
                        for req in reqs {
                            if !ids.insert(&req.id) {
                                // TODO turn this into a 4xx error after validating with data that this behavior isn't widely depended on
                                error!(
                                    "Duplicate RPC ID: {:?} for body {}",
                                    req.id,
                                    serde_json::to_string(&body).unwrap_or_default()
                                );
                            }
                        }
                    }

                    reqs.iter()
                        .map(|req| (req.id.to_string(), req.method.to_string()))
                        .collect()
                }
            };

            for (rpc_id, rpc_method) in rpcs {
                state.analytics.message(MessageInfo::new(
                    &query_params,
                    &headers,
                    query_params.session_id.clone(),
                    rpc_id,
                    rpc_method,
                    region.clone(),
                    country.clone(),
                    continent.clone(),
                    &provider.provider_kind(),
                    origin.clone(),
                    query_params.sdk_info.sv.clone(),
                    query_params.sdk_info.st.clone(),
                ));
            }
        }
        Err(e) => {
            // TODO turn this into a 4xx error after validating with data that this behavior isn't widely depended on
            error!(
                "TRIAGE: Invalid JSON-RPC request: {} for body: {}",
                e,
                String::from_utf8_lossy(&body)
            );
        }
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
        &provider.provider_kind(),
        response.status().as_u16(),
        Some(chain_id.clone()),
        None,
    );

    if provider.is_rate_limited(&mut response).await {
        state
            .metrics
            .add_rate_limited_call(provider.borrow(), project_id);
        *response.status_mut() = http::StatusCode::SERVICE_UNAVAILABLE;
    }

    state.metrics.add_external_http_latency(
        &provider.provider_kind(),
        external_call_start,
        Some(chain_id.clone()),
        None,
    );

    match response.status() {
        http::StatusCode::OK | http::StatusCode::BAD_REQUEST => {
            state
                .metrics
                .add_finished_provider_call(chain_id, provider.borrow());
        }
        _ => {
            error!(
                "Call to provider '{}' failed with status '{}' and body '{:?}'",
                provider.provider_kind(),
                response.status(),
                response.body()
            );
            state
                .metrics
                .add_failed_provider_call(chain_id, provider.borrow());
            *response.status_mut() = http::StatusCode::SERVICE_UNAVAILABLE;
        }
    };
    Ok(response)
}
