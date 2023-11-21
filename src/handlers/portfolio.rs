use {
    super::{PortfolioQueryParams, HANDLER_TASK_METRICS},
    crate::{error::RpcError, state::AppState},
    axum::{
        body::Bytes,
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        response::{Response, IntoResponse},
        Json,
    },
    ethers::abi::Address,
    hyper::HeaderMap,
    tap::TapFallible,
    tracing::log::error,
    std::{net::SocketAddr, sync::Arc},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<PortfolioQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    address: Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, path, headers, address, body)
        .with_metrics(HANDLER_TASK_METRICS.with_name("portfolio"))
        .await
}

async fn handler_internal(
    state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    query: Query<PortfolioQueryParams>,
    _path: MatchedPath,
    _headers: HeaderMap,
    Path(address): Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    let project_id = query.project_id.clone();
    let _address_hash = address.clone();
    address
        .parse::<Address>()
        .map_err(|_| RpcError::IdentityInvalidAddress)?;

    state.validate_project_access(&project_id).await?;

    let response = state
        .providers
        .portfolio_provider
        .get_portfolio(address, body, query.0)
        .await
        .tap_err(|e| {
            error!("Failed to call portfolio with {}", e);
        })?;

    Ok(Json(response).into_response())
}
