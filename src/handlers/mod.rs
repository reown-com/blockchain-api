use {
    crate::{analytics::MessageSource, error::RpcError, state::AppState, utils::network},
    axum::{
        extract::{MatchedPath, State},
        http::Request,
        middleware::Next,
        response::{IntoResponse, Response},
    },
    serde::{Deserialize, Serialize},
    std::{fmt::Display, sync::Arc, time::Instant},
    tracing::error,
    wc::metrics::TaskMetrics,
};

pub mod balance;
pub mod bundler;
pub mod chain_agnostic;
pub mod convert;
pub mod fungible_price;
pub mod generators;
pub mod health;
pub mod history;
pub mod identity;
pub mod metrics;
pub mod onramp;
pub mod portfolio;
pub mod profile;
pub mod proxy;
pub mod self_provider;
pub mod sessions;
pub mod supported_chains;
pub mod wallet;
pub mod ws_proxy;

static HANDLER_TASK_METRICS: TaskMetrics = TaskMetrics::new("handler_task");

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SdkInfoParams {
    pub st: Option<String>,
    pub sv: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcQueryParams {
    pub chain_id: String,
    pub project_id: String,
    /// Optional provider ID for the exact provider request
    pub provider_id: Option<String>,
    pub session_id: Option<String>,

    // TODO remove this param, as it can be set by actual rpc users but it shouldn't be
    /// Optional "source" field to indicate an internal request
    pub source: Option<MessageSource>,
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SupportedCurrencies {
    BTC,
    ETH,
    USD,
    EUR,
    GBP,
    AUD,
    CAD,
    INR,
    JPY,
}

impl Display for SupportedCurrencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SupportedCurrencies::BTC => "btc",
                SupportedCurrencies::ETH => "eth",
                SupportedCurrencies::USD => "usd",
                SupportedCurrencies::EUR => "eur",
                SupportedCurrencies::GBP => "gbp",
                SupportedCurrencies::AUD => "aud",
                SupportedCurrencies::CAD => "cad",
                SupportedCurrencies::INR => "inr",
                SupportedCurrencies::JPY => "jpy",
            }
        )
    }
}

/// Rate limit middleware that uses `rate_limiting`` token bucket sub crate
/// from the `utils-rs`. IP address and matched path are used as the token key.
pub async fn rate_limit_middleware<B>(
    State(state): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let headers = req.headers().clone();
    let ip = match network::get_forwarded_ip(&headers) {
        Some(ip) => ip.to_string(),
        None => {
            error!(
                "Failed to get forwarded IP from request in rate limit middleware. Skipping the \
                 rate-limiting."
            );
            // We are skipping the drop to the connect info IP address here, because we are
            // using the Load Balancer and if any issues with the X-Forwarded-IP header, we
            // will rate-limit the LB IP address.
            return next.run(req).await;
        }
    };
    let path = match req.extensions().get::<MatchedPath>().cloned() {
        Some(path) => path,
        None => {
            error!("Failed to get matched path from request in rate limit middleware");
            return next.run(req).await;
        }
    };
    // TODO: Get the project ID from the request path and add analytics for the
    // rate-limited requests for project ID.
    let project_id = None;

    let rate_limit = match state.rate_limit.as_ref() {
        Some(rate_limit) => rate_limit,
        None => {
            error!(
                "Rate limiting is not enabled in the state, but called in the rate limit \
                 middleware"
            );
            return next.run(req).await;
        }
    };

    let is_rate_limited_result = rate_limit
        .is_rate_limited(path.as_str(), &ip, project_id)
        .await;

    match is_rate_limited_result {
        Ok(_) => next.run(req).await,
        Err(e) => RpcError::from(e).into_response(),
    }
}

/// Endpoints latency and response status metrics middleware
pub async fn status_latency_metrics_middleware<B>(
    State(state): State<Arc<AppState>>,
    req: Request<B>,
    next: Next<B>,
) -> Response {
    // Extract the matched path from the request
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map_or("/unknown".to_string(), |mp| mp.as_str().to_string());
    let request_started = Instant::now();

    // Execute the request and get the response.
    let response = next.run(req).await;
    let request_latency = request_started.elapsed();

    // Record metrics async
    let state_clone = state.clone();
    let path_clone = path.clone();
    let status = response.status().as_u16();
    let latency_secs = request_latency.as_secs_f64();
    tokio::spawn(async move {
        state_clone
            .metrics
            .add_http_call(status, path_clone.clone());
        state_clone
            .metrics
            .add_http_latency(status, path_clone, latency_secs);
    });

    response
}
