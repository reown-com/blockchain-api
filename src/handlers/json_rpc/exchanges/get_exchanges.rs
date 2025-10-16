use {
    crate::handlers::json_rpc::exchanges::{
        get_enabled_features, get_feature_type, get_supported_exchanges,
        is_feature_enabled_for_project_id, Exchange, Feature, FeatureType,
    },
    crate::{handlers::SdkInfoParams, state::AppState},
    axum::{
        extract::{ConnectInfo, Query, State},
        Json,
    },
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, sync::Arc},
    thiserror::Error,
    tracing::debug,
    wc::metrics::{future_metrics, FutureExt},
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
    pub source: Option<String>,
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
    #[error("Unable to get enabled features: {0}")]
    UnableToGetEnabledFeatures(String),
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
    let feature_type = get_feature_type(query.source.as_deref());
    let project_features = get_enabled_features(state.clone(), &project_id)
        .await
        .map_err(|e| {
            GetExchangesError::InternalError(GetExchangesInternalError::UnableToGetEnabledFeatures(
                e.to_string(),
            ))
        })?;

    is_feature_enabled_for_project_id(state.clone(), &project_id, &project_features, &feature_type)
        .await
        .map_err(|e| GetExchangesError::ValidationError(e.to_string()))?;
    handler_internal(
        state,
        connect_info,
        headers,
        query,
        request,
        &project_features,
        &feature_type,
    )
    .with_metrics(future_metrics!("handler_task", "name" => "pay_get_exchanges"))
    .await
}

async fn handler_internal(
    _state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    _query: Query<QueryParams>,
    request: GetExchangesRequest,
    project_features: &[Feature],
    feature_type: &FeatureType,
) -> Result<GetExchangesResponse, GetExchangesError> {
    let all_exchanges =
        match get_supported_exchanges(request.asset.clone(), feature_type, project_features) {
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
