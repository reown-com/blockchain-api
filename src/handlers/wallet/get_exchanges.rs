use {
    crate::handlers::wallet::exchanges::{
        get_supported_exchanges, is_feature_enabled_for_project_id, Exchange,
    },
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
    tracing::debug,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetExchangesRequest {
    pub page: usize,
    #[serde(default)]
    pub include_only: Option<Vec<String>>,
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
    #[serde(default)]
    pub asset: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetExchangesResponse {
    pub total: usize,
    pub exchanges: Vec<Exchange>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
    pub source: Option<String>
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

impl GetExchangesError {
    pub fn is_internal(&self) -> bool {
        matches!(self, GetExchangesError::InternalError(_))
    }
}

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
    Json(request): Json<GetExchangesRequest>,
) -> Result<GetExchangesResponse, GetExchangesError> {
    is_feature_enabled_for_project_id(state.clone(), &project_id, query.source.as_deref())
        .await
        .map_err(|e| GetExchangesError::ValidationError(e.to_string()))?;
    handler_internal(state, connect_info, headers, query, request)
        .with_metrics(HANDLER_TASK_METRICS.with_name("pay_get_exchanges"))
        .await
}

async fn handler_internal(
    _state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    _query: Query<QueryParams>,
    request: GetExchangesRequest,
) -> Result<GetExchangesResponse, GetExchangesError> {
    let all_exchanges = match get_supported_exchanges(request.asset.clone()) {
        Ok(exchanges) => exchanges,
        Err(err) => {
            debug!(
                "Error getting supported exchanges: {:?}, asset: {:?}",
                err, request.asset
            );
            return Ok(GetExchangesResponse {
                total: 0,
                exchanges: vec![],
            });
        }
    };

    let exchanges = match (&request.include_only, &request.exclude) {
        (Some(_), Some(_)) => {
            return Err(GetExchangesError::ValidationError(
                "includeOnly and exclude are mutually exclusive".to_string(),
            ));
        }
        (Some(include_only), None) => all_exchanges
            .into_iter()
            .filter(|exchange| include_only.contains(&exchange.id))
            .collect(),
        (None, Some(exclude)) => all_exchanges
            .into_iter()
            .filter(|exchange| !exclude.contains(&exchange.id))
            .collect(),
        _ => all_exchanges,
    };

    Ok(GetExchangesResponse {
        total: exchanges.len(),
        exchanges,
    })
}
