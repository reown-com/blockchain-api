use {
    crate::{error::RpcError, handlers::HANDLER_TASK_METRICS, state::AppState},
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub r#type: PropertyType,
    pub project_id: String,
    pub countries: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum PropertyType {
    Countries,
    CryptoCurrencies,
    FiatCurrencies,
    PaymentMethods,
    FiatPurchasesLimits,
    CountriesDefaults,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<QueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("onramp_providers_properties"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    query: Query<QueryParams>,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query.project_id)
        .await?;

    let providers_properties = state
        .providers
        .onramp_multi_provider
        .get_providers_properties(query.0, state.metrics.clone())
        .await
        .tap_err(|e| {
            error!("Failed to call onramp providers properties with {}", e);
        })?;

    Ok(Json(providers_properties).into_response())
}
