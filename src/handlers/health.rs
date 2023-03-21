use {
    crate::state::AppState,
    axum::{extract::State, response::IntoResponse},
    hyper::StatusCode,
    std::sync::Arc,
};

pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        format!("OK v{}", state.compile_info.build().version()),
    )
}
