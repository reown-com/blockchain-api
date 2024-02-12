use {
    super::{
        super::HANDLER_TASK_METRICS,
        utils::{is_name_format_correct, is_name_in_allowed_zones, is_name_length_correct},
        ALLOWED_ZONES,
    },
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
    // Check if the name is in the correct format
    if !is_name_format_correct(&name) {
        return Err(RpcError::InvalidNameFormat(name));
    }

    // Check if the name length is correct
    if !is_name_length_correct(&name) {
        return Err(RpcError::InvalidNameLength(name));
    }

    // Check is name in the allowed zones
    if !is_name_in_allowed_zones(&name, &ALLOWED_ZONES) {
        return Err(RpcError::InvalidNameZone(name));
    }

    match get_name_and_addresses_by_name(name.clone(), &state.postgres).await {
        Ok(response) => Ok(Json(response).into_response()),
        Err(e) => match e {
            SqlxError::RowNotFound => return Err(RpcError::NameNotFound(name)),
            _ => {
                // Handle other types of errors
                error!("Failed to lookup name: {}", e);
                return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
            }
        },
    }
}
