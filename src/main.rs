use std::net::SocketAddr;
use std::sync::Arc;

use build_info::BuildInfo;
use dotenv::dotenv;
use hyper::Client;
use hyper_tls::HttpsConnector;
use opentelemetry::metrics::MeterProvider;
use tracing::info;
use warp::Filter;

use crate::env::Config;
use crate::metrics::Metrics;
use crate::project::Registry;
use crate::providers::ProviderRepository;
use crate::providers::{InfuraProvider, PoktProvider};
use crate::state::State;

mod env;
mod error;
mod handlers;
mod metrics;
mod project;
mod providers;
mod state;
mod storage;

#[tokio::main]
async fn main() -> error::RpcResult<()> {
    dotenv().ok();
    let config =
        Config::from_env().expect("Failed to load config, please ensure all env vars are defined.");

    let prometheus_exporter = opentelemetry_prometheus::exporter().init();
    let meter = prometheus_exporter
        .provider()
        .unwrap()
        .meter("rpc-proxy", None);

    let metrics = Metrics::new(&meter);
    let registry = Registry::new(&config.registry, &config.storage, &meter)?;
    let providers = init_providers(&config);

    let state = state::new_state(config, prometheus_exporter, metrics.clone(), registry);

    let port = state.config.server.port;
    let host = state.config.server.host.clone();
    let build_version = state.build_info.crate_info.version.clone();

    let state_arc = Arc::new(state);

    let state_filter = warp::any().map(move || state_arc.clone());

    let route_health = warp::get()
        .and(warp::path!("health"))
        .and(state_filter.clone())
        .and_then(handlers::health::handler);

    let route_metrics = warp::any()
        .and(warp::path!("metrics"))
        .and(state_filter.clone())
        .and_then(handlers::metrics::handler);

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
        .with(cors)
        .with(warp::log::custom(move |info| {
            let status = info.status().as_u16();
            let latency = info.elapsed().as_secs_f64();
            metrics.add_http_call(status, "proxy");
            metrics.add_http_latency(status, "proxy", latency);
        }));

    let routes = warp::any()
        .and(route_health)
        .or(proxy)
        .or(route_metrics)
        .with(warp::trace::request());

    info!("v{}", build_version);
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid socket address");
    warp::serve(routes).run(addr).await;

    Ok(())
}

fn init_providers(config: &Config) -> ProviderRepository {
    let infura_project_id = config.infura.project_id.clone();
    let infura_supported_chains = config.infura.supported_chains.clone();
    let pokt_project_id = config.pokt.project_id.clone();
    let pokt_supported_chains = config.pokt.supported_chains.clone();

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

    providers
}
