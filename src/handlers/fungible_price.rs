use {
    super::HANDLER_TASK_METRICS,
    crate::{error::RpcError, state::AppState, utils::crypto},
    axum::{
        extract::State,
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::{fmt::Display, sync::Arc},
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PriceCurrencies {
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

impl Display for PriceCurrencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PriceCurrencies::BTC => "btc",
            PriceCurrencies::ETH => "eth",
            PriceCurrencies::USD => "usd",
            PriceCurrencies::EUR => "eur",
            PriceCurrencies::GBP => "gbp",
            PriceCurrencies::AUD => "aud",
            PriceCurrencies::CAD => "cad",
            PriceCurrencies::INR => "inr",
            PriceCurrencies::JPY => "jpy",
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceQueryParams {
    pub project_id: String,
    pub currency: PriceCurrencies,
    pub addresses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponseBody {
    pub fungibles: Vec<FungiblePriceItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FungiblePriceItem {
    pub name: String,
    pub symbol: String,
    pub icon_url: String,
    pub price: f64,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    Json(query): Json<PriceQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("fungible_price"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: PriceQueryParams,
) -> Result<Response, RpcError> {
    let project_id = query.project_id.clone();
    state.validate_project_access_and_quota(&project_id).await?;

    if query.addresses.is_empty() && query.addresses.len() > 1 {
        return Err(RpcError::InvalidAddress);
    }
    let address = if let Some(address) = query.addresses.first() {
        address
    } else {
        return Err(RpcError::InvalidAddress);
    };

    let (_, chain_id, address) = crypto::disassemble_caip10(address)?;

    let response = state
        .providers
        .fungible_price_provider
        .get_price(
            &chain_id,
            &address,
            &query.currency,
            state.http_client.clone(),
        )
        .await
        .tap_err(|e| {
            error!("Failed to call fungible price with {}", e);
        })?;

    Ok(Json(response).into_response())
}
