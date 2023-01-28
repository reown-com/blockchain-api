use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::Context;
use env::BinanceConfig;
use env::ZKSyncConfig;
use error::RpcResult;
use hyper::Client;
use hyper_tls::HttpsConnector;
use opentelemetry::metrics::MeterProvider;
use providers::{ZKSyncProvider, BinanceProvider, InfuraProvider, PoktProvider, ProviderRepository};
use tokio::select;
use tokio::sync::broadcast;
use tracing::info;
use warp::Filter;

use crate::env::Config;
use crate::metrics::Metrics;
use crate::project::Registry;

mod analytics;
pub mod env;
pub mod error;
mod handlers;
mod json_rpc;
mod metrics;
mod project;
mod providers;
mod state;
mod storage;
mod utils;

pub async fn bootstrap(mut shutdown: broadcast::Receiver<()>, config: Config) -> RpcResult<()> {
    let prometheus_exporter = opentelemetry_prometheus::exporter().init();
    let meter = prometheus_exporter
        .provider()
        .unwrap()
        .meter("rpc-proxy", None);

    let metrics = Metrics::new(&meter);
    let registry = Registry::new(&config.registry, &config.storage, &meter)?;
    let providers = init_providers(&config);

    let external_ip = config
        .server
        .external_ip()
        .unwrap_or_else(|_| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

    let analytics = analytics::RPCAnalytics::new(&config.analytics, external_ip)
        .await
        .context("failed to init analytics")?;

    let state = state::new_state(
        config,
        providers,
        prometheus_exporter,
        metrics.clone(),
        registry,
        analytics,
    );

    let port = state.config.server.port;
    let host = state.config.server.host.clone();
    let build_version = state.compile_info.build().version();

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
        .and(warp::filters::addr::remote())
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

    select! {
    _ = warp::serve(routes).run(addr) => info!("Server starting"),
    _ = shutdown.recv() => info!("Shutdown signal received, killing servers"),
        }
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
        client: forward_proxy_client.clone(),
        project_id: pokt_project_id,
        supported_chains: pokt_supported_chains,
    };
    providers.add_provider("pokt".into(), Arc::new(pokt_provider));

    let binance_config = BinanceConfig::default();
    let binance_provider = BinanceProvider {
        client: forward_proxy_client.clone(),
        project_id: binance_config.project_id,
        supported_chains: binance_config.supported_chains,
    };
    providers.add_provider("binance".into(), Arc::new(binance_provider));

    let zksync_config = ZKSyncConfig::default();
    let zksync_provider = ZKSyncProvider {
        client: forward_proxy_client.clone(),
        project_id: zksync_config.project_id,
        supported_chains: zksync_config.supported_chains,
    };
    providers.add_provider("zksync".into(), Arc::new(zksync_provider));

    providers
}
