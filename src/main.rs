mod env;
mod error;
mod handlers;
mod providers;
mod state;

use crate::env::Config;
use build_info::BuildInfo;
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use crate::providers::InfuraProvider;
use crate::providers::ProviderRepository;
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
    let host = state.config.host.clone();
    let build_version = state.build_info.crate_info.version.clone();

    let state_arc = Arc::new(state);
    let infura_project_id = state_arc.config.infura_project_id.clone();
    let state_filter = warp::any().map(move || state_arc.clone());

    let health = warp::get()
        .and(warp::path!("health"))
        .and(state_filter.clone())
        .and_then(handlers::health::handler);

    let mut providers = ProviderRepository::default();
    let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let infura_provider = InfuraProvider {
        client: forward_proxy_client,
        infura_project_id,
    };
    providers.add_provider("eth".into(), Arc::new(infura_provider));
    let provider_filter = warp::any().map(move || providers.clone());

    let proxy = warp::any()
        .and(warp::path!("v1"))
        .and(provider_filter.clone())
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
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid socket address");
    warp::serve(routes).run(addr).await;

    Ok(())
}
