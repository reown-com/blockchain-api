use {
    crate::{error::RpcError, handlers::HANDLER_TASK_METRICS, state::AppState},
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, sync::Arc},
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub countries: Option<String>,
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersResponse {
    pub categories: Vec<String>,
    pub category_statuses: HashMap<String, String>,
    pub logos: Logos,
    pub name: String,
    pub service_provider: String,
    pub status: String,
    pub website_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Logos {
    pub dark: String,
    pub dark_short: String,
    pub light: String,
    pub light_short: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query: Query<QueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("onramp_providers"))
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

    let buy_options = state
        .providers
        .onramp_multi_provider
        .get_providers(query.0, state.metrics.clone())
        .await
        .tap_err(|e| {
            error!("Failed to call onramp providers with {}", e);
        })?;

    Ok(Json(buy_options).into_response())
}
