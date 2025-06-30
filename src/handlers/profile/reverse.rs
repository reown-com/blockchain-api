use {
    super::{super::HANDLER_TASK_METRICS, LookupQueryParams, EMPTY_RESPONSE},
    crate::{
        database::helpers::{get_name_and_addresses_by_name, get_names_by_address},
        error::RpcError,
        state::AppState,
    },
    axum::{
        extract::{Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::StatusCode,
    std::sync::Arc,
    tracing::log::error,
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    query: Query<LookupQueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("reverse_lookup"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
    query: Query<LookupQueryParams>,
) -> Result<Response, RpcError> {
    let names = match get_names_by_address(address, &state.postgres).await {
        Ok(names) => names,
        Err(e) => {
            error!("Error on get names by address: {e}");
            return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
        }
    };

    if names.is_empty() {
        // Return `HTTP 404` by default and an empty array for the future v2 support
        if query.api_version == Some(2) {
            return Ok(Json(EMPTY_RESPONSE).into_response());
        } else {
            return Err(RpcError::NameByAddressNotFound);
        }
    }

    let mut result = Vec::new();
    for name in names {
        match get_name_and_addresses_by_name(name.name, &state.postgres).await {
            Ok(response) => result.push(response),
            Err(e) => {
                // Unexpected behavior when looking up a name for an address
                error!("Unexpected behavior when looking up a name for an address: {e}");
                return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
            }
        }
    }
    Ok(Json(result).into_response())
}
