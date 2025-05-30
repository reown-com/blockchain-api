use axum::{routing::get, Router};
use tower_service::Service;
use worker::*;

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
    "Hello Axum!"
}
