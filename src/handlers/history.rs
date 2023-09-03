use {
    super::{
        HistoryResponseBody,
        HistoryTransaction,
        HistoryTransactionMetadata,
        ZerionResponseBody,
        HANDLER_TASK_METRICS,
    },
    crate::{
        error::{RpcError, RpcResult},
        handlers::HistoryQueryParams,
        state::AppState,
    },
    axum::{
        body::Bytes,
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::abi::Address,
    futures_util::StreamExt,
    hyper::{http, Client, HeaderMap},
    hyper_tls::HttpsConnector,
    std::{net::SocketAddr, sync::Arc},
    tap::TapFallible,
    tracing::{
        info,
        log::error,
    },
    wc::future::FutureExt,
};

// TODO: move this into a provider package
async fn get_zerion_transactions(
    address: String,
    body: Bytes,
    api_key: String,
) -> RpcResult<HistoryResponseBody> {
    // TODO: query parameters
    let uri = format!(
        "https://api.zerion.io/v1/wallets/{}/transactions/?currency=usd",
        address
    );

    let hyper_request = hyper::http::Request::builder()
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("authorization", format!("Basic {}", api_key))
        .body(hyper::body::Body::from(body))?;

    let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let response = forward_proxy_client.request(hyper_request).await?;

    if response.status() != http::StatusCode::OK {
        return Err(RpcError::TransactionProviderError);
    }

    // Now parse this hyper::Body into a ZerionResponseBody
    let mut body = response.into_body();
    let mut bytes = Vec::new();
    while let Some(next) = body.next().await {
        bytes.extend_from_slice(&next?);
    }
    let body: ZerionResponseBody = serde_json::from_slice(&bytes)?;

    let transactions: Vec<HistoryTransaction> = body
        .data
        .into_iter()
        .map(|f| HistoryTransaction {
            id: f.id,
            metadata: HistoryTransactionMetadata {
                operation_type: f.attributes.operation_type,
                hash: f.attributes.hash,
                mined_at: f.attributes.mined_at,
                nonce: f.attributes.nonce,
                sent_from: f.attributes.sent_from,
                sent_to: f.attributes.sent_to,
                status: f.attributes.status,
            },
        })
        .collect();

    Ok(HistoryResponseBody { data: transactions })
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<HistoryQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    address: Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, path, headers, address, body)
        .with_metrics(HANDLER_TASK_METRICS.with_name("transactions"))
        .await
}

// TODO: parse and use query parameters
async fn handler_internal(
    state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    _query: Query<HistoryQueryParams>,
    _path: MatchedPath,
    _headers: HeaderMap,
    Path(address): Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    let _address = address
        .parse::<Address>()
        .map_err(|_| RpcError::IdentityInvalidAddress)?;

    let project = state
        .registry
        .project_data(&query.project_id)
        .await
        .tap_err(|_| state.metrics.add_rejected_project())?;

    project
        .validate_access(&query.project_id, None)
        .tap_err(|e| {
            state.metrics.add_rejected_project();
            info!(
                "Denied access for project: {}, with reason: {}",
                query.project_id, e
            );
        })?;

    let project_id = query.project_id.clone();

    let zerion_api_key = std::env::var("RPC_PROXY_ZERION_API_KEY")
        .expect("Missing RPC_PROXY_INFURA_PROJECT_ID env var");

    let response = get_zerion_transactions(address, body, zerion_api_key)
        .await
        .tap_err(|e| {
            error!("Failed call history with {}", e);
        })?;

    Ok(Json(response).into_response())
}
