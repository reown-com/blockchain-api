use prometheus_core::TextEncoder;
use std::sync::Arc;
use tracing::error;
use warp::http;

use crate::state::State;

pub async fn handler(state: Arc<State>) -> Result<impl warp::Reply, warp::Rejection> {
    let data = state.exporter.registry().gather();
    match TextEncoder::new().encode_to_string(&data) {
        Ok(content) => {
            let response = warp::reply::with_status(content, http::StatusCode::OK);

            Ok(response)
        }
        Err(e) => {
            error!(?e, "Failed to parse metrics");
            let response = warp::reply::with_status(
                "Failed to get metrics".into(),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            );

            Ok(response)
        }
    }
}
