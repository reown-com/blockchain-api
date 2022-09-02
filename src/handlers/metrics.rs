use crate::State;
use prometheus_core::TextEncoder;
use std::sync::Arc;
use tracing::error;
use warp::http;

pub async fn handler(state: Arc<State>) -> Result<impl warp::Reply, warp::Rejection> {
    match &state.exporter {
        None => {
            let response = warp::reply::with_status(
                format!(
                    "No metrics installed v{}",
                    state.build_info.crate_info.version
                ),
                http::StatusCode::OK,
            );

            Ok(response)
        }
        Some(exporter) => {
            let data = exporter.registry().gather();
            match TextEncoder::new().encode_to_string(&data) {
                Ok(content) => {
                    let response =
                        warp::reply::with_status(format!("{}", content), http::StatusCode::OK);

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
    }
}
