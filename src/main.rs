mod env;
mod error;
mod handlers;
mod state;

use crate::env::Config;
use build_info::BuildInfo;
use dotenv::dotenv;
use tracing::info;
use std::sync::Arc;

use crate::state::{State};

use warp::Filter;

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

    let forward_proxy_client = hyper::Client::new();
    
    let proxy = warp::get()
        .and(warp::path!("v1"))
        .and(state_filter.clone())
        .and(warp::any().map(move || forward_proxy_client.clone()))
        .and(warp::method())
        .and(warp::path::full())
        .and(
            warp::filters::query::raw()
                .or(warp::any().map(|| String::default()))
                .unify(),
        )
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(handlers::proxy::handler);

    let routes = warp::any()
        .and(health,)
        .or(proxy,)
        .with(warp::trace::request());

    info!("v{}", build_version);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;

    Ok(())
}
