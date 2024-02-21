use {
    super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, state::AppState},
    axum::{
        body::Bytes,
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::abi::Address,
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, sync::Arc},
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioQueryParams {
    pub project_id: String,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioResponseBody {
    pub data: Vec<PortfolioPosition>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioPosition {
    pub id: String,
    pub name: String,
    pub symbol: String,
}

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

#[tracing::instrument(skip_all)]
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
        .map_err(|_| RpcError::InvalidAddress)?;

    state.validate_project_access_and_quota(&project_id).await?;

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
