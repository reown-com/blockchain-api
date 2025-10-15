use {
    crate::{
        handlers::{
            json_rpc::exchanges::{
                get_enabled_features,
                get_exchange_by_id,
                get_feature_type,
                is_feature_enabled_for_project_id,
                transactions::{
                    mark_failed as mark_transaction_failed,
                    mark_succeeded as mark_transaction_succeeded,
                    touch_pending as touch_pending_transaction,
                },
                BuyTransactionStatus,
                ExchangeError,
                Feature,
                FeatureType,
                GetBuyStatusParams,
            },
            SdkInfoParams,
        },
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
    wc::metrics::{future_metrics, FutureExt},
};

const MAX_SESSION_ID_LENGTH: usize = 50;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetExchangeBuyStatusRequest {
    pub exchange_id: String,
    pub session_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetExchangeBuyStatusResponse {
    pub status: BuyTransactionStatus,
    pub tx_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
    pub source: Option<String>,
}

#[derive(Error, Debug)]
pub enum GetExchangeBuyStatusError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Exchange not found: {0}")]
    ExchangeNotFound(String),

    #[error("Session not found or expired: {0}")]
    SessionNotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl GetExchangeBuyStatusError {
    pub fn is_internal(&self) -> bool {
        matches!(self, GetExchangeBuyStatusError::InternalError(_))
    }
}

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    _connect_info: ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    query: Query<QueryParams>,
    Json(request): Json<GetExchangeBuyStatusRequest>,
) -> Result<GetExchangeBuyStatusResponse, GetExchangeBuyStatusError> {
    let feature_type = get_feature_type(query.source.as_deref());
    let project_features = get_enabled_features(state.clone(), &project_id)
        .await
        .map_err(|e| GetExchangeBuyStatusError::InternalError(e.to_string()))?;

    is_feature_enabled_for_project_id(state.clone(), &project_id, &project_features, &feature_type)
        .await
        .map_err(|e| GetExchangeBuyStatusError::ValidationError(e.to_string()))?;
    handler_internal(state, project_id, request, &project_features, &feature_type)
        .with_metrics(future_metrics!("handler_task", "name" => "pay_get_exchange_buy_status"))
        .await
}

async fn handler_internal(
    state: State<Arc<AppState>>,
    project_id: String,
    request: GetExchangeBuyStatusRequest,
    project_features: &[Feature],
    feature_type: &FeatureType,
) -> Result<GetExchangeBuyStatusResponse, GetExchangeBuyStatusError> {
    let exchange = get_exchange_by_id(&request.exchange_id, feature_type, project_features)
        .map_err(|e| GetExchangeBuyStatusError::ExchangeNotFound(e.to_string()))?;

    if request.session_id.is_empty() || request.session_id.len() > MAX_SESSION_ID_LENGTH {
        return Err(GetExchangeBuyStatusError::ValidationError(
            "Invalid session ID".to_string(),
        ));
    }

    let arc_state = state.0.clone();
    let result = exchange
        .get_buy_status(State(arc_state), GetBuyStatusParams {
            project_id,
            session_id: request.session_id.clone(),
        })
        .await;

    match result {
        Ok(response) => {
            match response.status {
                BuyTransactionStatus::Success => {
                    let _ = mark_transaction_succeeded(
                        &state,
                        &request.session_id,
                        &request.exchange_id,
                        response.tx_hash.as_deref(),
                    )
                    .await;
                }
                BuyTransactionStatus::Failed => {
                    let _ = mark_transaction_failed(
                        &state,
                        &request.session_id,
                        &request.exchange_id,
                        Some("provider_failed"),
                        response.tx_hash.as_deref(),
                    )
                    .await;
                }
                _ => {
                    let _ = touch_pending_transaction(
                        &state,
                        &request.exchange_id,
                        &request.session_id,
                    )
                    .await;
                }
            }

            Ok(GetExchangeBuyStatusResponse {
                status: response.status,
                tx_hash: response.tx_hash,
            })
        }
        Err(e) => match e {
            ExchangeError::ValidationError(msg) => {
                Err(GetExchangeBuyStatusError::ValidationError(msg))
            }
            _ => {
                debug!(
                    error = %e,
                    session_id = %request.session_id,
                    exchange_id = %request.exchange_id,
                    "Internal error, unable to get exchange buy status"
                );
                Err(GetExchangeBuyStatusError::InternalError(format!(
                    "Unable to get exchange buy status: {e:?}"
                )))
            }
        },
    }
}
