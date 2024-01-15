use {
    super::super::HANDLER_TASK_METRICS,
    crate::{database::helpers::get_name_and_addresses_by_name, error::RpcError, state::AppState},
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::StatusCode,
    sqlx::Error as SqlxError,
    std::sync::Arc,
    tracing::log::error,
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    name: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, name)
        .with_metrics(HANDLER_TASK_METRICS.with_name("profile"))
        .await
}

#[tracing::instrument(skip(state))]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Response, RpcError> {
    match get_name_and_addresses_by_name(name, &state.postgres).await {
        Ok(response) => Ok(Json(response).into_response()),
        Err(e) => match e {
            SqlxError::RowNotFound => {
                // Respond with a "Not Found" status code if name was found
                let not_found_response = (StatusCode::NOT_FOUND, "Name is not registered");
                Ok(not_found_response.into_response())
            }
            _ => {
                // Handle other types of errors
                error!("Failed to lookup name: {}", e);
                return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
            }
        },
    }
}
