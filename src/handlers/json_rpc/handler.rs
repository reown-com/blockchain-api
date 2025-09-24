use {
    super::{
        exchanges::{
            get_exchange_buy_status::{self, GetExchangeBuyStatusError},
            get_exchange_url::{self, GetExchangeUrlError},
            get_exchanges::{self, GetExchangesError},
        },
        pos::{self, BuildPosTxsError, CheckPosTxError, SupportedNetworksError},
        wallet::{
            get_assets::{self, GetAssetsError},
            get_calls_status::QueryParams as CallStatusQueryParams,
            get_calls_status::{self, GetCallsStatusError},
            prepare_calls::{self, PrepareCallsError},
            send_prepared_calls::{self, SendPreparedCallsError},
        },
    },
    crate::{
        error::RpcError,
        handlers::SdkInfoParams,
        json_rpc::{ErrorResponse, JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcResult},
        state::AppState,
        utils::simple_request_json::SimpleRequestJson,
    },
    axum::extract::{ConnectInfo, Query},
    axum::response::{IntoResponse, Response},
    axum::{extract::State, Json},
    hyper::{HeaderMap, StatusCode},
    serde::Deserialize,
    std::net::SocketAddr,
    std::sync::Arc,
    std::time::Instant,
    thiserror::Error,
    tracing::error,
    wc::metrics::{future_metrics, FutureExt},
    yttrium::wallet_service_api,
};

/// CORS default allowed origins
const CORS_ALLOWED_ORIGINS: [&str; 1] = ["http://localhost:3000"];

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WalletQueryParams {
    pub project_id: String,
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
    pub source: Option<String>,
}

// TODO support batch requests (and validate unique RPC IDs)
pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<WalletQueryParams>,
    SimpleRequestJson(request_payload): SimpleRequestJson<JsonRpcRequest>,
) -> Response {
    handler_internal(state, connect_info, headers, query, request_payload)
        .with_metrics(future_metrics!("handler_task", "name" => "wallet"))
        .await
}

// Wrapper that adds dynamic CORS headers based on project registry data
pub async fn json_rpc_with_dynamic_cors(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<WalletQueryParams>,
    SimpleRequestJson(request_payload): SimpleRequestJson<JsonRpcRequest>,
) -> Response {
    let method_name = request_payload.method.clone();
    let mut response = handler(
        state.clone(),
        connect_info,
        headers.clone(),
        query.clone(),
        SimpleRequestJson(request_payload),
    )
    .await;

    // Add debug header listing allowed origins for this project
    if let Some(list) = get_project_allowed_origins(state.0.clone(), &query.project_id).await {
        insert_allowed_origins_debug_header(&mut response, &list);
    }

    // Apply CORS policy:
    // - For selected PAY_* methods: echo Origin only if it's allowed for the project
    // - For all other methods: allow all origins
    match method_name.as_ref() {
        PAY_GET_EXCHANGES | PAY_GET_EXCHANGE_URL | PAY_GET_EXCHANGE_BUY_STATUS => {
            if let Some(origin) = headers
                .get(hyper::header::ORIGIN)
                .and_then(|v| v.to_str().ok())
            {
                if is_origin_allowed_for_project(state.0.clone(), &query.project_id, origin).await {
                    insert_cors_headers(&mut response, origin);
                }
            }
        }
        _ => {
            insert_cors_allow_all_headers(&mut response);
        }
    }

    response
}

// OPTIONS preflight handler for /v1/json-rpc
pub async fn json_rpc_preflight(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WalletQueryParams>,
) -> Response {
    // Always allow preflight with wildcard; actual POST will enforce per-method policy
    let mut response = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(axum::body::Body::empty())
        .unwrap()
        .into_response();
    if let Some(list) = get_project_allowed_origins(state.clone(), &query.project_id).await {
        insert_allowed_origins_debug_header(&mut response, &list);
    }
    insert_cors_allow_all_headers(&mut response);
    response
}

async fn is_origin_allowed_for_project(
    state: Arc<AppState>,
    project_id: &str,
    origin: &str,
) -> bool {
    // Try to fetch project data; on registry unavailability, disallow by default
    let Ok(project) = state.registry.project_data(project_id).await else {
        return false;
    };

    let origin_lc = origin.to_ascii_lowercase();

    // Allow default allowed origins by default
    if CORS_ALLOWED_ORIGINS
        .iter()
        .any(|o| o.eq_ignore_ascii_case(&origin_lc))
    {
        return true;
    }
    // Parse origin host if possible
    let origin_host = url::Url::parse(origin)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()));

    // Check exact origins first
    if project
        .data
        .allowed_origins
        .iter()
        .any(|o| o.eq_ignore_ascii_case(&origin_lc))
    {
        return true;
    }

    // Check hostname against allowed_origins
    if let Some(host) = origin_host {
        if project
            .data
            .allowed_origins
            .iter()
            .any(|o| !o.contains("://") && o.eq_ignore_ascii_case(&host))
        {
            return true;
        }
    }

    false
}

fn insert_cors_headers(response: &mut Response, origin: &str) {
    let headers = response.headers_mut();
    // Strip CR/LF to avoid header injection
    let cleaned_origin = origin.replace(['\r', '\n'], "");
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        match hyper::header::HeaderValue::from_str(&cleaned_origin) {
            Ok(value) => value,
            Err(e) => {
                // Don't set CORS headers for invalid origins
                error!("Invalid origin header value: {origin}, {e}");
                return;
            }
        },
    );
    headers.insert(
        hyper::header::VARY,
        hyper::header::HeaderValue::from_static("Origin"),
    );
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
        hyper::header::HeaderValue::from_static("POST, OPTIONS"),
    );
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
        hyper::header::HeaderValue::from_static(
            "content-type, user-agent, referer, origin, access-control-request-method, access-control-request-headers, solana-client, sec-fetch-mode, x-sdk-type, x-sdk-version",
        ),
    );
}

fn insert_cors_allow_all_headers(response: &mut Response) {
    let headers = response.headers_mut();
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        hyper::header::HeaderValue::from_static("*"),
    );
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
        hyper::header::HeaderValue::from_static("POST, OPTIONS"),
    );
    headers.insert(
        hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
        hyper::header::HeaderValue::from_static(
            "content-type, user-agent, referer, origin, access-control-request-method, access-control-request-headers, solana-client, sec-fetch-mode, x-sdk-type, x-sdk-version",
        ),
    );
}

async fn get_project_allowed_origins(
    state: Arc<AppState>,
    project_id: &str,
) -> Option<Vec<String>> {
    let project = state.registry.project_data(project_id).await.ok()?;
    let mut allowed_origins: Vec<String> = Vec::new();
    allowed_origins.extend(project.data.allowed_origins.into_iter());
    // Deduplicate, case-insensitive
    allowed_origins.sort_by_key(|s| s.to_ascii_lowercase());
    allowed_origins.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    // Append default allowed origins
    allowed_origins.extend(CORS_ALLOWED_ORIGINS.iter().map(|s| s.to_string()));
    Some(allowed_origins)
}

fn insert_allowed_origins_debug_header(response: &mut Response, list: &[String]) {
    // Sanitize each origin by stripping CR/LF and keeping only valid header values
    let sanitized: Vec<String> = list
        .iter()
        .map(|s| s.replace(['\r', '\n'], ""))
        .filter(|s| hyper::header::HeaderValue::from_str(s).is_ok())
        .collect();

    if sanitized.is_empty() {
        return;
    }
    if let Ok(value) = hyper::header::HeaderValue::from_str(&sanitized.join(",")) {
        response.headers_mut().insert(
            hyper::header::HeaderName::from_static("x-allowed-origins"),
            value,
        );
    }
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<WalletQueryParams>,
    request: JsonRpcRequest,
) -> Response {
    let start = Instant::now();
    let method = request.method.as_ref().to_string();

    let result = handle_rpc(
        state.clone(),
        connect_info,
        headers,
        query,
        request.method.clone(),
        request.params,
    )
    .await;

    let (response, json_rpc_code) = match result {
        Ok(result) => {
            let response = Json(JsonRpcResponse::Result(JsonRpcResult::new(
                request.id, result,
            )))
            .into_response();
            (response, 0)
        }
        Err(e) => {
            let code = e.to_json_rpc_error_code();
            let json = Json(JsonRpcResponse::Error(JsonRpcError::new(
                request.id,
                ErrorResponse {
                    code,
                    message: e.to_string().into(),
                    data: None,
                },
            )));
            let response = if e.is_internal() {
                error!("Internal server error handling wallet RPC request: {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, json).into_response()
            } else {
                (StatusCode::BAD_REQUEST, json).into_response()
            };
            (response, code)
        }
    };

    let state_clone = state.clone();
    let latency = start.elapsed();
    tokio::spawn(async move {
        state_clone
            .metrics
            .add_json_rpc_call(method.clone(), json_rpc_code);
        state_clone
            .metrics
            .add_json_rpc_call_latency(method, latency);
    });

    response
}

pub const WALLET_PREPARE_CALLS: &str = "wallet_prepareCalls";
pub const WALLET_SEND_PREPARED_CALLS: &str = "wallet_sendPreparedCalls";
pub const WALLET_GET_CALLS_STATUS: &str = "wallet_getCallsStatus";
pub const PAY_GET_EXCHANGES: &str = "reown_getExchanges";
pub const PAY_GET_EXCHANGE_URL: &str = "reown_getExchangePayUrl";
pub const PAY_GET_EXCHANGE_BUY_STATUS: &str = "reown_getExchangeBuyStatus";
pub const POS_BUILD_TRANSACTIONS: &str = "wc_pos_buildTransactions";
pub const POS_CHECK_TRANSACTION: &str = "wc_pos_checkTransaction";
pub const POS_SUPPORTED_NETWORKS: &str = "wc_pos_supportedNetworks";

#[derive(Debug, Error)]
enum Error {
    #[error("Invalid project ID: {0}")]
    InvalidProjectId(RpcError),

    #[error("{WALLET_PREPARE_CALLS}: {0}")]
    PrepareCalls(PrepareCallsError),

    #[error("{WALLET_SEND_PREPARED_CALLS}: {0}")]
    SendPreparedCalls(SendPreparedCallsError),

    #[error("{WALLET_GET_CALLS_STATUS}: {0}")]
    GetCallsStatus(GetCallsStatusError),

    #[error("{PAY_GET_EXCHANGES}: {0}")]
    GetExchanges(GetExchangesError),

    #[error("{PAY_GET_EXCHANGE_URL}: {0}")]
    GetUrl(GetExchangeUrlError),

    #[error("{}: {0}", wallet_service_api::WALLET_GET_ASSETS)]
    GetAssets(GetAssetsError),

    #[error("{PAY_GET_EXCHANGE_BUY_STATUS}: {0}")]
    GetExchangeBuyStatus(GetExchangeBuyStatusError),

    #[error("{POS_BUILD_TRANSACTIONS}: {0}")]
    PosBuildTransactions(#[source] BuildPosTxsError),

    #[error("{POS_CHECK_TRANSACTION}: {0}")]
    PosCheckTransaction(#[source] CheckPosTxError),

    #[error("{POS_SUPPORTED_NETWORKS}: {0}")]
    PosSupportedNetworks(#[source] SupportedNetworksError),

    #[error("Method not found")]
    MethodNotFound,

    #[error("Invalid params: {0}")]
    InvalidParams(serde_json::Error),

    #[error("Internal error")]
    Internal(InternalError),
}

#[derive(Debug, Error)]
enum InternalError {
    #[error("Serializing response: {0}")]
    SerializeResponse(serde_json::Error),
}

impl Error {
    fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            Error::InvalidProjectId(_) => -1,
            Error::PrepareCalls(_) => -2, // TODO more specific codes
            Error::SendPreparedCalls(_) => -3, // TODO more specific codes
            Error::GetCallsStatus(_) => -4, // TODO more specific codes
            Error::GetAssets(_) => -5,    // TODO more specific codes
            Error::GetExchanges(_) => -6,
            Error::GetUrl(_) => -7,
            Error::GetExchangeBuyStatus(_) => -8,
            // -18900 to -18999 reserved for POS
            Error::PosBuildTransactions(e) => e.to_json_rpc_error_code(),
            Error::PosCheckTransaction(e) => e.to_json_rpc_error_code(),
            Error::PosSupportedNetworks(e) => e.to_json_rpc_error_code(),
            Error::MethodNotFound => -32601,
            Error::InvalidParams(_) => -32602,
            Error::Internal(_) => -32000,
        }
    }

    fn is_internal(&self) -> bool {
        match self {
            Error::InvalidProjectId(_) => false,
            Error::PrepareCalls(e) => e.is_internal(),
            Error::SendPreparedCalls(e) => e.is_internal(),
            Error::GetCallsStatus(e) => e.is_internal(),
            Error::GetAssets(e) => e.is_internal(),
            Error::GetExchanges(e) => e.is_internal(),
            Error::GetUrl(e) => e.is_internal(),
            Error::GetExchangeBuyStatus(e) => e.is_internal(),
            Error::PosBuildTransactions(e) => e.is_internal(),
            Error::PosCheckTransaction(e) => e.is_internal(),
            Error::PosSupportedNetworks(e) => e.is_internal(),
            Error::MethodNotFound => false,
            Error::InvalidParams(_) => false,
            Error::Internal(_) => true,
        }
    }
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handle_rpc(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<WalletQueryParams>,
    method: Arc<str>,
    params: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    let project_id = query.project_id;
    state
        .validate_project_access_and_quota(&project_id)
        .await
        // TODO refactor to differentiate between user and server errors
        .map_err(Error::InvalidProjectId)?;

    match method.as_ref() {
        WALLET_PREPARE_CALLS => serde_json::to_value(
            &prepare_calls::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
            )
            .await
            .map_err(Error::PrepareCalls)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        WALLET_SEND_PREPARED_CALLS => serde_json::to_value(
            &send_prepared_calls::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
            )
            .await
            .map_err(Error::SendPreparedCalls)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        WALLET_GET_CALLS_STATUS => serde_json::to_value(
            &get_calls_status::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
                connect_info,
                headers,
                Query(CallStatusQueryParams {
                    sdk_info: query.sdk_info,
                }),
            )
            .await
            .map_err(Error::GetCallsStatus)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        wallet_service_api::WALLET_GET_ASSETS => serde_json::to_value(
            &get_assets::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
                connect_info,
                headers,
                Query(get_assets::QueryParams {
                    sdk_info: query.sdk_info,
                }),
            )
            .await
            .map_err(Error::GetAssets)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        PAY_GET_EXCHANGES => serde_json::to_value(
            &get_exchanges::handler(
                state,
                project_id,
                connect_info,
                headers,
                Query(get_exchanges::QueryParams {
                    sdk_info: query.sdk_info,
                    source: query.source,
                }),
                Json(serde_json::from_value(params).map_err(Error::InvalidParams)?),
            )
            .await
            .map_err(Error::GetExchanges)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        PAY_GET_EXCHANGE_URL => serde_json::to_value(
            &get_exchange_url::handler(
                state,
                project_id,
                connect_info,
                headers,
                Query(get_exchange_url::QueryParams {
                    sdk_info: query.sdk_info,
                    source: query.source,
                }),
                Json(serde_json::from_value(params).map_err(Error::InvalidParams)?),
            )
            .await
            .map_err(Error::GetUrl)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        PAY_GET_EXCHANGE_BUY_STATUS => serde_json::to_value(
            &get_exchange_buy_status::handler(
                state,
                project_id,
                connect_info,
                headers,
                Query(get_exchange_buy_status::QueryParams {
                    sdk_info: query.sdk_info,
                    source: query.source,
                }),
                Json(serde_json::from_value(params).map_err(Error::InvalidParams)?),
            )
            .await
            .map_err(Error::GetExchangeBuyStatus)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        POS_BUILD_TRANSACTIONS => serde_json::to_value(
            &pos::build_transactions::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
            )
            .await
            .map_err(Error::PosBuildTransactions)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        POS_CHECK_TRANSACTION => serde_json::to_value(
            &pos::check_transaction::handler(
                state,
                project_id,
                serde_json::from_value(params).map_err(Error::InvalidParams)?,
            )
            .await
            .map_err(Error::PosCheckTransaction)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        POS_SUPPORTED_NETWORKS => serde_json::to_value(
            &pos::supported_networks::handler(state, project_id)
                .await
                .map_err(Error::PosSupportedNetworks)?,
        )
        .map_err(|e| Error::Internal(InternalError::SerializeResponse(e))),
        _ => Err(Error::MethodNotFound),
    }
}
