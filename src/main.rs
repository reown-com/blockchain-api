mod env;
mod error;
mod handlers;
mod providers;
mod state;

use crate::env::Config;
use crate::state::Metrics;
use build_info::BuildInfo;
use dotenv::dotenv;
use opentelemetry::metrics::MeterProvider;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use crate::providers::ProviderRepository;
use crate::providers::{InfuraProvider, PoktProvider};
use crate::state::State;

use warp::Filter;

use hyper::Client;
use hyper_tls::HttpsConnector;

#[tokio::main]
async fn main() -> error::RpcResult<()> {
    dotenv().ok();
    let config =
        env::get_config().expect("Failed to load config, please ensure all env vars are defined.");

    let prometheus_exporter = opentelemetry_prometheus::exporter().init();
    let meter = prometheus_exporter
        .provider()
        .unwrap()
        .meter("rpc-proxy", None);

    let rpc_call_counter = meter
        .u64_counter("rpc_call_counter")
        .with_description("The number of rpc calls served")
        .init();

    let state = state::new_state(config, prometheus_exporter, Metrics { rpc_call_counter });

    let port = state.config.port;
    let host = state.config.host.clone();
    let build_version = state.build_info.crate_info.version.clone();

    let state_arc = Arc::new(state);
    let infura_project_id = state_arc.config.infura_project_id.clone();
    let infura_supported_chains = state_arc.config.infura_supported_chains.clone();
    let pokt_project_id = state_arc.config.pokt_project_id.clone();
    let pokt_supported_chains = state_arc.config.pokt_supported_chains.clone();
    let state_filter = warp::any().map(move || state_arc.clone());

    let health = warp::get()
        .and(warp::path!("health"))
        .and(state_filter.clone())
        .and_then(handlers::health::handler);

    let metrics = warp::any()
        .and(warp::path!("metrics"))
        .and(state_filter.clone())
        .and_then(handlers::metrics::handler);

    let mut providers = ProviderRepository::default();
    let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let infura_provider = InfuraProvider {
        client: forward_proxy_client.clone(),
        project_id: infura_project_id,
        supported_chains: infura_supported_chains,
    };
    providers.add_provider("infura".into(), Arc::new(infura_provider));
    let pokt_provider = PoktProvider {
        client: forward_proxy_client,
        project_id: pokt_project_id,
        supported_chains: pokt_supported_chains,
    };
    providers.add_provider("pokt".into(), Arc::new(pokt_provider));
    let provider_filter = warp::any().map(move || providers.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "User-Agent",
            "Content-Type",
            "Sec-Fetch-Mode",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "solana-client",
        ])
        .allow_methods(vec!["GET", "POST"]);
    let proxy = warp::any()
        .and(warp::path!("v1"))
        .and(state_filter.clone())
        .and(provider_filter.clone())
        .and(warp::method())
        .and(warp::path::full())
        .and(warp::filters::query::query())
        .and(warp::header::headers_cloned())
        .and(warp::body::bytes())
        .and_then(handlers::proxy::handler)
        .with(cors);

    let routes = warp::any()
        .and(health)
        .or(proxy)
        .or(metrics)
        .with(warp::trace::request());

    info!("v{}", build_version);
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid socket address");
    warp::serve(routes).run(addr).await;

    Ok(())
}
