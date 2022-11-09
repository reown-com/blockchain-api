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
use crate::project::Registry;
use crate::providers::ProviderRepository;
use crate::providers::{InfuraProvider, PoktProvider};
use crate::state::Metrics;
use crate::state::State;

mod env;
mod error;
mod handlers;
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

    let rpc_call_counter = meter
        .u64_counter("rpc_call_counter")
        .with_description("The number of rpc calls served")
        .init();

    let http_call_counter = meter
        .u64_counter("http_call_counter")
        .with_description("The number of http calls served")
        .init();

    let http_latency_tracker = meter
        .f64_counter("http_latency_tracker")
        .with_description("The http call latency")
        .init();

    let http_call_counter_arc = Arc::new(http_call_counter.clone());
    let http_latency_tracker_arc = Arc::new(http_latency_tracker.clone());

    let registry = Registry::new(&config.registry, &config.storage, &meter)?;

    let state = state::new_state(
        config,
        prometheus_exporter,
        Metrics {
            rpc_call_counter,
            http_call_counter,
        },
        registry,
    );

    let port = state.config.server.port;
    let host = state.config.server.host.clone();
    let build_version = state.build_info.crate_info.version.clone();

    let state_arc = Arc::new(state);
    let infura_project_id = state_arc.config.infura.project_id.clone();
    let infura_supported_chains = state_arc.config.infura.supported_chains.clone();
    let pokt_project_id = state_arc.config.pokt.project_id.clone();
    let pokt_supported_chains = state_arc.config.pokt.supported_chains.clone();

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
        .with(cors)
        .with(warp::log::custom(move |info| {
            let status = info.status().as_u16();
            let latency = info.elapsed().as_secs_f64();
            http_call_counter_arc.add(
                1,
                &[
                    opentelemetry::KeyValue::new("code", i64::from(status)),
                    opentelemetry::KeyValue::new("route", "proxy"),
                ],
            );
            http_latency_tracker_arc.add(
                latency,
                &[
                    opentelemetry::KeyValue::new("code", i64::from(status)),
                    opentelemetry::KeyValue::new("route", "proxy"),
                ],
            )
        }));

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
