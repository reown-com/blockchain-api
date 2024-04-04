use {
    super::{
        super::HANDLER_TASK_METRICS,
        utils::{is_name_format_correct, is_name_registered},
        ALLOWED_ZONES,
    },
    crate::{error::RpcError, state::AppState, utils::suggestions::dictionary_suggestions},
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    wc::future::FutureExt,
};

const SUGGESTION_OPTIONS: usize = 5;
const MIN_NAME_LENGTH: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameSuggestionsResponse {
    pub suggestions: Vec<NameSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameSuggestion {
    pub name: String,
    pub registered: bool,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    name: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, name)
        .with_metrics(HANDLER_TASK_METRICS.with_name("name_suggestions"))
        .await
}

#[tracing::instrument(skip(state))]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Response, RpcError> {
    if name.len() < MIN_NAME_LENGTH {
        return Err(RpcError::InvalidNameLength(name));
    }
    if !is_name_format_correct(&name) {
        return Err(RpcError::InvalidNameFormat(name));
    }

    let mut suggestions = Vec::new();
    let candidates = dictionary_suggestions(&name);

    // Adding the exact match for each of available zones to check if it is
    // registered
    for zone in ALLOWED_ZONES.iter() {
        let exact_name_with_zone = format!("{}.{}", name, zone);
        suggestions.push(NameSuggestion {
            name: exact_name_with_zone.clone(),
            registered: is_name_registered(exact_name_with_zone, &state.postgres).await,
        });
    }

    // Iterate found dictionary candidates and check if they are registered
    for suggested_name in candidates {
        // Get name suggestion for each of available zones if the name is free
        for zone in ALLOWED_ZONES.iter() {
            let name_with_zone = format!("{}.{}", suggested_name, zone);
            let is_registered = is_name_registered(name_with_zone.clone(), &state.postgres).await;

            if !is_registered {
                suggestions.push(NameSuggestion {
                    name: name_with_zone,
                    registered: false,
                });
            }
        }
        if suggestions.len() == SUGGESTION_OPTIONS {
            break;
        }
    }

    Ok(Json(NameSuggestionsResponse { suggestions }).into_response())
}
