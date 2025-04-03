use {
    crate::handlers::wallet::exchanges::{ExchangeError, ExchangeType, GetBuyUrlParams},
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
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

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
    Json(request): Json<GeneratePayUrlRequest>,
) -> Result<GeneratePayUrlResponse, GetExchangeUrlError> {
    handler_internal(state, connect_info, headers, query, request)
        .with_metrics(HANDLER_TASK_METRICS.with_name("pay_get_exchange_url"))
        .await
}

async fn handler_internal(
    state: State<Arc<AppState>>,
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

    let amount = match usize::from_str_radix(request.amount.trim_start_matches("0x"), 16) {
        Ok(amount) => amount,
        Err(_) => {
            return Err(GetExchangeUrlError::ValidationError(format!(
                "Invalid amount. Expected a valid hexadecimal number: {}",
                request.amount
            )))
        }
    };

    let result = exchange
        .get_buy_url(
            state,
            GetBuyUrlParams {
                asset,
                amount,
                recipient: address,
            },
        )
        .await;

    match result {
        Ok(url) => Ok(GeneratePayUrlResponse { url }),
        Err(e) => match e {
            ExchangeError::ValidationError(msg) => Err(GetExchangeUrlError::ValidationError(msg)),
            _ => {
                debug!(
                    error = %e,
                    "Internal error, unable to get exchange URL"
                );
                Err(GetExchangeUrlError::InternalError(
                    "Unable to get exchange URL".to_string(),
                ))
            }
        },
    }
}
