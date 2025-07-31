use {
    crate::{
        error::RpcError,
        handlers::{generators::GeneratorQueryParams, HANDLER_TASK_METRICS},
        state::AppState,
    },
    axum::{
        body::Bytes,
        extract::{ConnectInfo, MatchedPath, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::{HeaderMap, StatusCode},
    serde::{Deserialize, Serialize},
    std::{fmt, net::SocketAddr, sync::Arc},
    tracing::log::debug,
    validator::Validate,
    wc::future::FutureExt,
};

/// Request according to the Coinbase Pay SDK `generateOnRampURL`
/// https://docs.cloud.coinbase.com/pay-sdk/docs/generating-url#generateonrampurl-parameters
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct OnRampURLRequest {
    #[serde(skip_deserializing)]
    pub app_id: String,
    #[validate(length(min = 1))]
    pub destination_wallets: Vec<DestinationWallet>,
    #[validate(length(min = 32, max = 50))]
    pub partner_user_id: String,
    pub default_network: Option<String>,
    pub preset_crypto_amount: Option<usize>,
    pub preset_fiat_amount: Option<usize>,
    pub default_experience: Option<ExperienceType>,
    pub handling_requested_urls: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ExperienceType {
    Send,
    Buy,
}

impl fmt::Display for ExperienceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExperienceType::Send => write!(f, "send"),
            ExperienceType::Buy => write!(f, "buy"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DestinationWallet {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_networks: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OnRampURLResponse {
    pub url: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    addr: ConnectInfo<SocketAddr>,
    query_params: Query<GeneratorQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RpcError> {
    handler_internal(state, addr, query_params, path, headers, body)
        .with_metrics(HANDLER_TASK_METRICS.with_name("onrampurl"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Query(query_params): Query<GeneratorQueryParams>,
    _path: MatchedPath,
    _headers: HeaderMap,
    body: Bytes,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query_params.project_id)
        .await?;

    let parameters = match serde_json::from_slice::<OnRampURLRequest>(&body) {
        Ok(parameters) => parameters,
        Err(e) => {
            debug!("Error deserializing request body: {e}");
            return Ok((
                StatusCode::BAD_REQUEST,
                "Error deserializing request body: {}",
            )
                .into_response());
        }
    };

    let cb_provider = &state.providers.onramp_provider;
    let on_ramp_url = cb_provider
        .generate_on_ramp_url(parameters, state.metrics.clone())
        .await?;

    Ok(Json(OnRampURLResponse { url: on_ramp_url }).into_response())
}
