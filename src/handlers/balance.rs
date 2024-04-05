use {
    super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, state::AppState},
    axum::{
        extract::{Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::abi::Address,
    serde::{Deserialize, Serialize},
    std::{fmt::Display, sync::Arc},
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BalanceCurrencies {
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

impl Display for BalanceCurrencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BalanceCurrencies::BTC => "btc",
                BalanceCurrencies::ETH => "eth",
                BalanceCurrencies::USD => "usd",
                BalanceCurrencies::EUR => "eur",
                BalanceCurrencies::GBP => "gbp",
                BalanceCurrencies::AUD => "aud",
                BalanceCurrencies::CAD => "cad",
                BalanceCurrencies::INR => "inr",
                BalanceCurrencies::JPY => "jpy",
            }
        )
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQueryParams {
    pub project_id: String,
    pub currency: BalanceCurrencies,
    pub chain_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponseBody {
    pub balances: Vec<BalanceItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceItem {
    pub name: String,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    pub price: f64,
    pub quantity: BalanceQuantity,
    pub icon_url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceQuantity {
    pub decimals: String,
    pub numeric: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<BalanceQueryParams>,
    address: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, query, address)
        .with_metrics(HANDLER_TASK_METRICS.with_name("balance"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<BalanceQueryParams>,
    Path(address): Path<String>,
) -> Result<Response, RpcError> {
    let project_id = query.project_id.clone();
    address
        .parse::<Address>()
        .map_err(|_| RpcError::InvalidAddress)?;

    state.validate_project_access_and_quota(&project_id).await?;

    let response = state
        .providers
        .balance_provider
        .get_balance(address, query.0)
        .await
        .tap_err(|e| {
            error!("Failed to call balance with {}", e);
        })?;

    Ok(Json(response).into_response())
}
