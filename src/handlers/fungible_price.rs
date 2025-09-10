use {
    super::SupportedCurrencies,
    crate::{
        error::RpcError,
        state::AppState,
        utils::{crypto, simple_request_json::SimpleRequestJson},
    },
    axum::{
        extract::State,
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    tap::TapFallible,
    tracing::log::error,
    wc::metrics::{future_metrics, FutureExt},
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceQueryParams {
    pub project_id: String,
    pub currency: SupportedCurrencies,
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
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub icon_url: String,
    pub price: f64,
    pub decimals: u8,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    SimpleRequestJson(query): SimpleRequestJson<PriceQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query)
        .with_metrics(future_metrics!("handler_task", "name" => "fungible_price"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
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

    let (namespace, chain_id, address) = crypto::disassemble_caip10(address)?;
    if !crypto::is_address_valid(&address, &namespace) {
        return Err(RpcError::InvalidAddress);
    }

    let provider = state
        .providers
        .fungible_price_providers
        .get(&namespace)
        .ok_or_else(|| RpcError::UnsupportedNamespace(namespace))?;

    let response = provider
        .get_price(
            &chain_id,
            &address,
            &query.currency,
            &state.providers.token_metadata_cache,
            state.metrics.clone(),
        )
        .await
        .tap_err(|e| {
            error!("Failed to call fungible price with {e}");
        })?;

    Ok(Json(response).into_response())
}
