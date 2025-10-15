use {
    super::SuggestionsParams,
    crate::{
        error::RpcError,
        names::{
            suggestions::dictionary_suggestions,
            utils::{is_name_format_correct, is_name_registered},
        },
        state::AppState,
    },
    axum::{
        extract::{Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    wc::metrics::{future_metrics, FutureExt},
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
    query: Query<SuggestionsParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, name, query)
        .with_metrics(future_metrics!("handler_task", "name" => "name_suggestions"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(query): Query<SuggestionsParams>,
) -> Result<Response, RpcError> {
    if name.len() < MIN_NAME_LENGTH {
        return Err(RpcError::InvalidNameLength(name));
    }
    if !is_name_format_correct(&name) {
        return Err(RpcError::InvalidNameFormat(name));
    }

    let mut suggestions = Vec::new();
    let candidates = dictionary_suggestions(&name);

    // Use the `zone` query parameter if it is provided for the new AppKit versions
    // Otherwise, use the first zone in the allowed zones list for the backward
    // compatibility with the old AppKit versions
    let allowed_zones = state.config.names.allowed_zones.as_ref().ok_or_else(|| {
        RpcError::InvalidConfiguration("Names allowed zones are not defined".to_string())
    })?;
    let default_zone = allowed_zones.first().ok_or_else(|| {
        RpcError::InvalidConfiguration("Names allowed zones are empty".to_string())
    })?;
    let zone = query.zone.unwrap_or_else(|| default_zone.to_string());

    // Adding the exact match for the main zone to check if it is
    // registered
    let exact_name_with_zone = format!("{name}.{zone}");
    suggestions.push(NameSuggestion {
        name: exact_name_with_zone.clone(),
        registered: is_name_registered(exact_name_with_zone, &state.postgres).await,
    });

    // Iterate found dictionary candidates and check if they are registered
    for suggested_name in candidates {
        // Get name suggestion for the main zone if the name is free
        let name_with_zone = format!("{suggested_name}.{zone}");
        let is_registered = is_name_registered(name_with_zone.clone(), &state.postgres).await;

        if !is_registered {
            suggestions.push(NameSuggestion {
                name: name_with_zone,
                registered: false,
            });
        }
        if suggestions.len() == SUGGESTION_OPTIONS {
            break;
        }
    }

    Ok(Json(NameSuggestionsResponse { suggestions }).into_response())
}
