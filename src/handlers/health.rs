use {
    crate::state::AppState,
    axum::{extract::State, response::IntoResponse},
    hyper::StatusCode,
    std::sync::Arc,
};

pub async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        format!(
            "OK v{}, commit hash: {}, features: {}, uptime: {:?} seconds",
            state.compile_info.build().version(),
            state.compile_info.git().short_hash(),
            state.compile_info.build().features(),
            state.uptime.elapsed().as_secs()
        ),
    )
}
