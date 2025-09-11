use {
    crate::handlers::wallet::exchanges::{
        is_feature_enabled_for_project_id, BuyTransactionStatus, ExchangeError, ExchangeType,
        GetBuyStatusParams,
        transactions::{
            mark_failed as mark_transaction_failed,
            mark_succeeded as mark_transaction_succeeded,
            touch_pending as touch_pending_transaction,
        },
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
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
    Json(request): Json<GetExchangeBuyStatusRequest>,
) -> Result<GetExchangeBuyStatusResponse, GetExchangeBuyStatusError> {
    is_feature_enabled_for_project_id(state.clone(), &project_id, query.source.as_deref())
        .await
        .map_err(|e| GetExchangeBuyStatusError::ValidationError(e.to_string()))?;
    handler_internal(state, connect_info, headers, query, request)
        .with_metrics(future_metrics!("handler_task", "name" => "pay_get_exchange_buy_status"))
        .await
}

async fn handler_internal(
    state: State<Arc<AppState>>,
    _connect_info: ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    _query: Query<QueryParams>,
    request: GetExchangeBuyStatusRequest,
) -> Result<GetExchangeBuyStatusResponse, GetExchangeBuyStatusError> {
    let exchange = ExchangeType::from_id(&request.exchange_id).ok_or_else(|| {
        GetExchangeBuyStatusError::ExchangeNotFound(format!(
            "Exchange {} not found",
            request.exchange_id
        ))
    })?;

    if request.session_id.is_empty() || request.session_id.len() > MAX_SESSION_ID_LENGTH {
        return Err(GetExchangeBuyStatusError::ValidationError(
            "Invalid session ID".to_string(),
        ));
    }

    let arc_state = state.0.clone();
    let result = exchange
        .get_buy_status(
            State(arc_state),
            GetBuyStatusParams {
                session_id: request.session_id.clone(),
            },
        )
        .await;

    match result {
        Ok(response) => {
            match response.status {
                BuyTransactionStatus::Success => {
                    let tx_hash = response.tx_hash.clone();
                    let _ = mark_transaction_succeeded(&state, &request.session_id, tx_hash.as_deref()).await;
                }
                BuyTransactionStatus::Failed => {
                    let tx_hash = response.tx_hash.clone();
                    let _ = mark_transaction_failed(
                        &state,
                        &request.session_id,
                        Some("provider_failed"),
                        tx_hash.as_deref(),
                    )
                    .await;
                }
                _ => {
                    let _ = touch_pending_transaction(&state, &request.session_id).await;
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
