mod env;
mod error;
mod handlers;
mod state;

use crate::env::Config;
use build_info::BuildInfo;
use dotenv::dotenv;
use std::sync::Arc;
use tracing::info;

use crate::state::State;

use warp::Filter;

use hyper::Client;
use hyper_tls::HttpsConnector;

#[tokio::main]
async fn main() -> error::Result<()> {
    dotenv().ok();
    let config =
        env::get_config().expect("Failed to load config, please ensure all env vars are defined.");

    let state = state::new_state(config);

    let port = state.config.port;
    let build_version = state.build_info.crate_info.version.clone();

    let state_arc = Arc::new(state);
    let state_filter = warp::any().map(move || state_arc.clone());

    let health = warp::get()
        .and(warp::path!("health"))
        .and(state_filter.clone())
        .and_then(handlers::health::handler);

    let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let proxy_client_filter = warp::any().map(move || forward_proxy_client.clone());

    let proxy = warp::any()
        .and(warp::path!("v1"))
        .and(state_filter.clone())
        .and(proxy_client_filter)
        .and(warp::method())
        .and(warp::path::full())
        .and(warp::filters::query::query())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(handlers::proxy::handler);

    let routes = warp::any()
        .and(health)
        .or(proxy)
        .with(warp::trace::request());

    info!("v{}", build_version);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;

    Ok(())
}
