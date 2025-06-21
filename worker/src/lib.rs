use axum::{routing::get, Router};
use tower_service::Service;
use worker::*;

pub mod active_config;
pub mod config;

fn router() -> Router {
    Router::new().route("/v1", get(proxy))
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router().call(req).await?)
}

pub async fn proxy() -> &'static str {
    // TODO validate rate limit, parse request, etc.
    // TODO call cachable function to determine which provider to use
    //      - returns the provider
    //      - if provider fails, call the function again but cache-bust w/ some parameter. This will return a different provider?
    "Hello Axum!"
}
