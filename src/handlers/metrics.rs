use {
    crate::state::AppState,
    axum::{extract::State, response::IntoResponse},
    hyper::StatusCode,
    prometheus_core::TextEncoder,
    std::sync::Arc,
    tracing::error,
};

pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let data = state.exporter.registry().gather();
    match TextEncoder::new().encode_to_string(&data) {
        Ok(content) => (StatusCode::OK, content),
        Err(e) => {
            error!(?e, "Failed to parse metrics");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get metrics".to_string(),
            )
        }
    }
}
