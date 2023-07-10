use {
    axum::response::IntoResponse,
    hyper::StatusCode,
    tracing::error,
    wc::metrics::ServiceMetrics,
};

pub async fn handler() -> impl IntoResponse {
    let result = ServiceMetrics::export();

    match result {
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
