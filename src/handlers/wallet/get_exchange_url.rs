use {
    crate::handlers::wallet::exchanges::{
        is_feature_enabled_for_project_id, ExchangeError, ExchangeType, GetBuyUrlParams,
    },
    crate::{
        handlers::{SdkInfoParams, HANDLER_TASK_METRICS},
        state::AppState,
        utils::crypto::{disassemble_caip10, Caip19Asset},
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
    uuid::Uuid,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePayUrlRequest {
    pub exchange_id: String,
    pub asset: String,
    pub amount: String,
    pub recipient: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePayUrlResponse {
    pub url: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
    pub source: Option<String>,
}

#[derive(Error, Debug)]
pub enum GetExchangeUrlError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Exchange not found: {0}")]
    ExchangeNotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl GetExchangeUrlError {
    pub fn is_internal(&self) -> bool {
        matches!(self, GetExchangeUrlError::InternalError(_))
    }
}

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
    Json(request): Json<GeneratePayUrlRequest>,
) -> Result<GeneratePayUrlResponse, GetExchangeUrlError> {
    is_feature_enabled_for_project_id(state.clone(), &project_id, query.source.as_deref())
        .await
        .map_err(|e| GetExchangeUrlError::ValidationError(e.to_string()))?;
    handler_internal(state, project_id, connect_info, headers, query, request)
        .with_metrics(HANDLER_TASK_METRICS.with_name("pay_get_exchange_url"))
        .await
}

async fn handler_internal(
    state: State<Arc<AppState>>,
    project_id: String,
    _connect_info: ConnectInfo<SocketAddr>,
    _headers: HeaderMap,
    _query: Query<QueryParams>,
    request: GeneratePayUrlRequest,
) -> Result<GeneratePayUrlResponse, GetExchangeUrlError> {
    let exchange = ExchangeType::from_id(&request.exchange_id).ok_or_else(|| {
        GetExchangeUrlError::ExchangeNotFound(format!("Exchange {} not found", request.exchange_id))
    })?;

    let asset = Caip19Asset::parse(&request.asset)
        .map_err(|e| GetExchangeUrlError::ValidationError(e.to_string()))?;

    let (namespace, chain_id, address) = disassemble_caip10(&request.recipient)
        .map_err(|e| GetExchangeUrlError::ValidationError(e.to_string()))?;
    if namespace.to_string() != asset.chain_id().namespace() {
        return Err(GetExchangeUrlError::ValidationError(format!(
            "Invalid recipient. CAIP-10 namespace must match asset namespace: {} != {}",
            namespace,
            asset.asset_namespace()
        )));
    }
    if chain_id != asset.chain_id().reference() {
        return Err(GetExchangeUrlError::ValidationError(format!(
            "Invalid recipient. CAIP-10 chainId must match asset chainId: {} != {}",
            chain_id,
            asset.asset_id()
        )));
    }

    if !exchange.is_asset_supported(&asset) {
        return Err(GetExchangeUrlError::ValidationError(format!(
            "Asset {} is not supported by exchange {}",
            asset, request.exchange_id
        )));
    }

    // support decimal and hex
    let amount = match request.amount.parse::<f64>() {
        Ok(parsed_amount) => parsed_amount,
        Err(_) => match usize::from_str_radix(request.amount.trim_start_matches("0x"), 16) {
            Ok(parsed_hex_amount) => parsed_hex_amount as f64,
            Err(_) => {
                return Err(GetExchangeUrlError::ValidationError(format!(
                    "Invalid amount. Expected a valid number or hexadecimal string: {}",
                    request.amount
                )));
            }
        },
    };

    // Removing dashes from the session id because binance only accepts alphanumeric characters
    let session_id = Uuid::new_v4().to_string().replace("-", "");

    let result = exchange
        .get_buy_url(
            state,
            GetBuyUrlParams {
                project_id,
                asset,
                amount,
                recipient: address,
                session_id: session_id.clone(),
            },
        )
        .await;

    match result {
        Ok(url) => Ok(GeneratePayUrlResponse { url, session_id }),
        Err(e) => match e {
            ExchangeError::ValidationError(msg) => Err(GetExchangeUrlError::ValidationError(msg)),
            _ => {
                debug!(
                    error = %e,
                    "Internal error, unable to get exchange URL"
                );
                Err(GetExchangeUrlError::InternalError(format!(
                    "Unable to get exchange URL: {e:?}"
                )))
            }
        },
    }
}
