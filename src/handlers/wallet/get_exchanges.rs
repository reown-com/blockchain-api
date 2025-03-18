use {
    crate::handlers::wallet::exchanges::{get_supported_exchanges, Exchange},
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
    #[error("Validation error: {0}")]
    ValidationError(String),

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
        .with_metrics(HANDLER_TASK_METRICS.with_name("pay_get_exchanges"))
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
    let mut exchanges = get_supported_exchanges();

    match (&request.include_only, &request.exclude) {
        (Some(_), Some(_)) => {
            return Err(GetExchangesError::ValidationError(
                "includeOnly and exclude are mutually exclusive".to_string(),
            ));
        }
        (Some(include_only), None) => {
            exchanges.retain(|exchange| include_only.contains(&exchange.id));
        }
        (None, Some(exclude)) => {
            exchanges.retain(|exchange| !exclude.contains(&exchange.id));
        }
        _ => {}
    }

    Ok(GetExchangesResponse {
        total: exchanges.len() as u32,
        exchanges,
    })
}
