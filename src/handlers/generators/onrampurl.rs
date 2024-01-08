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
    tracing::log::{error, info},
    url::Url,
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

const CB_PAY_HOST: &str = "https://pay.coinbase.com";
const CB_PAY_PATH: &str = "/buy/select-asset";

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

    let cb_app_id = match state.config.providers.clone().coinbase_app_id {
        Some(app_id) => app_id,
        None => {
            error!("Coinbase App ID is not configured");
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
        }
    };

    let mut parameters = match serde_json::from_slice::<OnRampURLRequest>(&body) {
        Ok(parameters) => parameters,
        Err(e) => {
            info!("Error deserializing request body: {}", e);
            return Ok((
                StatusCode::BAD_REQUEST,
                "Error deserializing request body: {}",
            )
                .into_response());
        }
    };
    parameters.app_id = cb_app_id;

    let on_ramp_url = match generate_on_ramp_url(CB_PAY_HOST, CB_PAY_PATH, parameters) {
        Ok(on_ramp_url) => on_ramp_url,
        Err(e) => {
            error!("Error generating on-ramp URL: {}", e);
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
        }
    };

    Ok(Json(OnRampURLResponse { url: on_ramp_url }).into_response())
}

pub fn generate_on_ramp_url(
    host: &str,
    path: &str,
    parameters: OnRampURLRequest,
) -> Result<String, anyhow::Error> {
    let mut url = Url::parse(host)?;
    url.set_path(path);

    // Required parameters
    url.query_pairs_mut()
        .append_pair("appId", &parameters.app_id);
    url.query_pairs_mut().append_pair(
        "destinationWallets",
        &serde_json::to_string(&parameters.destination_wallets)?,
    );
    url.query_pairs_mut()
        .append_pair("partnerUserId", &parameters.partner_user_id);

    // Optional parameters
    if let Some(default_network) = parameters.default_network {
        url.query_pairs_mut()
            .append_pair("defaultNetwork", &default_network);
    }
    if let Some(preset_crypto_amount) = parameters.preset_crypto_amount {
        url.query_pairs_mut()
            .append_pair("presetCryptoAmount", &preset_crypto_amount.to_string());
    }
    if let Some(preset_fiat_amount) = parameters.preset_fiat_amount {
        url.query_pairs_mut()
            .append_pair("presetFiatAmount", &preset_fiat_amount.to_string());
    }
    if let Some(default_experience) = parameters.default_experience {
        url.query_pairs_mut()
            .append_pair("defaultExperience", &default_experience.to_string());
    }
    if let Some(handling_requested_urls) = parameters.handling_requested_urls {
        url.query_pairs_mut().append_pair(
            "handlingRequestedUrls",
            &handling_requested_urls.to_string(),
        );
    }

    Ok(url.to_string())
}

#[test]
fn ensure_generate_on_ramp_url() {
    let app_id = "CB_TEST_APP_ID".to_string();
    let address = "0x1234567890123456789012345678901234567890".to_string();
    let partner_user_id = "1234567890123456789012345678901234567890".to_string();

    let parameters = OnRampURLRequest {
        app_id: app_id.clone(),
        destination_wallets: vec![DestinationWallet {
            address: address.clone(),
            blockchains: None,
            assets: None,
            supported_networks: None,
        }],
        partner_user_id: partner_user_id.clone(),
        default_network: None,
        preset_crypto_amount: None,
        preset_fiat_amount: None,
        default_experience: None,
        handling_requested_urls: None,
    };

    let url =
        Url::parse(&generate_on_ramp_url(CB_PAY_HOST, CB_PAY_PATH, parameters).unwrap()).unwrap();

    assert_eq!(url.scheme(), "https");
    assert_eq!(
        url.host_str().unwrap(),
        Url::parse(CB_PAY_HOST).unwrap().host_str().unwrap()
    );
    assert_eq!(url.path(), CB_PAY_PATH);
    assert_eq!(
        url.query_pairs().find(|(key, _)| key == "appId").unwrap().1,
        app_id
    );
    assert_eq!(
        url.query_pairs()
            .find(|(key, _)| key == "destinationWallets")
            .unwrap()
            .1,
        format!("[{{\"address\":\"{}\"}}]", address)
    );
    assert_eq!(
        url.query_pairs()
            .find(|(key, _)| key == "partnerUserId")
            .unwrap()
            .1,
        partner_user_id
    );
}
