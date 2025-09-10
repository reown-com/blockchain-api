use {
    crate::{error::RpcError, state::AppState, utils::simple_request_json::SimpleRequestJson},
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub project_id: String,
    pub session_data: SessionData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionData {
    pub country_code: Option<String>,
    pub destination_currency_code: String,
    pub lock_fields: Option<Vec<String>>,
    pub payment_method_type: Option<String>,
    pub redirect_url: Option<String>,
    pub service_provider: String,
    pub source_amount: f64,
    pub source_currency_code: String,
    pub wallet_address: String,
    pub wallet_tag: Option<String>,
    pub additional_params: Option<AdditionalParams>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalParams {
    pub nft_checkout: bool,
    pub nft_description: String,
    pub nft_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WidgetResponse {
    pub widget_url: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    SimpleRequestJson(request_payload): SimpleRequestJson<QueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, request_payload)
        .with_metrics(future_metrics!("handler_task", "name" => "onramp_widget"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    request_payload: QueryParams,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&request_payload.project_id)
        .await?;

    let widget_response = state
        .providers
        .onramp_multi_provider
        .get_widget(request_payload, state.metrics.clone())
        .await
        .tap_err(|e| {
            error!("Failed to call onramp widget with {e}");
        })?;

    Ok(Json(widget_response).into_response())
}
