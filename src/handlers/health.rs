use std::sync::Arc;
use warp::http;

use crate::state::State;

pub async fn handler(state: Arc<State>) -> Result<impl warp::Reply, warp::Rejection> {
    let response = warp::reply::with_status(
        format!("OK v{}", state.compile_info.build().version()),
        http::StatusCode::OK,
    );

    Ok(response)
}
