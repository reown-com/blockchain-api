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
        utils::{cors, cors::CORS_ALLOWED_ORIGINS, simple_request_json::SimpleRequestJson},
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
    if let Some(list) = cors::get_project_allowed_origins(state.0.clone(), &query.project_id).await
    {
        cors::insert_allowed_origins_debug_header(&mut response, &list);
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
                    cors::insert_cors_headers(&mut response, origin);
                }
            }
        }
        _ => {
            cors::insert_cors_allow_all_headers(&mut response);
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
    if let Some(list) = cors::get_project_allowed_origins(state.clone(), &query.project_id).await {
        cors::insert_allowed_origins_debug_header(&mut response, &list);
    }
    cors::insert_cors_allow_all_headers(&mut response);
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
    // Parse origin URL details if possible
    let parsed_origin = url::Url::parse(origin).ok();
    let origin_host = parsed_origin
        .as_ref()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()));
    let origin_scheme = parsed_origin
        .as_ref()
        .map(|u| u.scheme().to_ascii_lowercase());
    let origin_effective_port: Option<u16> = {
        fn default_port_for_scheme(s: &str) -> Option<u16> {
            match s {
                "http" => Some(80),
                "https" => Some(443),
                _ => None,
            }
        }
        match (&parsed_origin, &origin_scheme) {
            (Some(u), Some(s)) => u.port().or_else(|| default_port_for_scheme(s)),
            _ => None,
        }
    };

    // Single-pass matcher over allowed entries
    let origin_allowed = project.data.allowed_origins.iter().any(|entry| {
        let entry_lc = entry.trim().to_ascii_lowercase();

        // Fast path: exact origin string match
        if entry_lc == origin_lc {
            return true;
        }

        // Full origin pattern with scheme
        if let Some((scheme_pat, rest)) = entry_lc.split_once("://") {
            // Scheme must match
            if origin_scheme.as_deref() != Some(scheme_pat) {
                return false;
            }

            // Extract host[:port] (ignore any path if present)
            let host_port = rest.split('/').next().unwrap_or("");
            if host_port.is_empty() {
                return false;
            }
            let (host_pat, port_pat_opt) = host_port
                .split_once(':')
                .map(|(h, p)| (h, Some(p)))
                .unwrap_or((host_port, None));

            let Some(ref host_lc) = origin_host else {
                return false;
            };
            if !cors::host_matches_pattern(host_pat, host_lc) {
                return false;
            }

            // If port is specified in entry, it must match effective origin port
            if let Some(port_s) = port_pat_opt {
                if let Ok(port_num) = port_s.parse::<u16>() {
                    return origin_effective_port.is_some_and(|p| p == port_num);
                }
                return false;
            }
            return true;
        }

        // Host-only entry (wildcard supported)
        if let Some(ref host_lc) = origin_host {
            return cors::host_matches_pattern(&entry_lc, host_lc);
        }
        false
    });

    if origin_allowed {
        return true;
    }

    false
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
