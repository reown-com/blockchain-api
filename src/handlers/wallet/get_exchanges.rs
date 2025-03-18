use {
    crate::{
        handlers::{SdkInfoParams, HANDLER_TASK_METRICS},
        state::AppState,
    },
    axum::{
        extract::{ConnectInfo, Query, State},
        Json,
    },
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, sync::Arc},
    thiserror::Error,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetExchangesRequest {
    pub page: u32,
    pub include_only: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetExchangesResponse {
    pub total: u32,
    pub exchanges: Vec<Exchange>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

#[derive(Error, Debug)]
pub enum GetExchangesError {
    #[error("Internal error")]
    InternalError(GetExchangesInternalError),
}

#[derive(Error, Debug)]
pub enum GetExchangesInternalError {
    #[error("Internal error")]
    InternalError(String),
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
    Json(request): Json<GetExchangesRequest>,
) -> Result<GetExchangesResponse, GetExchangesError> {
    handler_internal(state, connect_info, headers, query, request)
        .with_metrics(HANDLER_TASK_METRICS.with_name("wallet_get_exchanges"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
    request: GetExchangesRequest,
) -> Result<GetExchangesResponse, GetExchangesError> {
    // For now, return hardcoded response
    Ok(GetExchangesResponse {
        total: 2,
        exchanges: vec![
            Exchange {
                id: "binance".to_string(),
                name: "Binance".to_string(),
                image_url: Some("https://example.com/binance-logo.png".to_string()),
            },
            Exchange {
                id: "coinbase".to_string(),
                name: "Coinbase".to_string(),
                image_url: Some("https://example.com/coinbase-logo.png".to_string()),
            },
        ],
    })
}
