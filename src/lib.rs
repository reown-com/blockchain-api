use {
    crate::{env::Config, metrics::Metrics, project::Registry},
    anyhow::Context,
    axum::{
        http,
        response::Response,
        routing::{any, get},
        Router,
    },
    env::{
        BinanceConfig,
        InfuraConfig,
        OmniatechConfig,
        PoktConfig,
        PublicnodeConfig,
        ZKSyncConfig,
    },
    error::RpcResult,
    hyper::{header::HeaderName, Client},
    hyper_tls::HttpsConnector,
    opentelemetry::metrics::MeterProvider,
    providers::{
        BinanceProvider,
        InfuraProvider,
        InfuraWsProvider,
        OmniatechProvider,
        PoktProvider,
        ProviderRepository,
        PublicnodeProvider,
        ZKSyncProvider,
    },
    std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
        time::Duration,
    },
    tokio::{select, sync::broadcast},
    tower::ServiceBuilder,
    tower_http::{
        cors::{Any, CorsLayer},
        trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    },
    tracing::{info, Level, Span},
};

mod analytics;
pub mod env;
pub mod error;
mod extractors;
mod handlers;
mod json_rpc;
mod metrics;
mod project;
mod providers;
mod state;
mod storage;
mod utils;
mod ws;

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

    let cors = CorsLayer::new().allow_origin(Any).allow_headers([
        http::header::CONTENT_TYPE,
        http::header::USER_AGENT,
        http::header::REFERER,
        http::header::ORIGIN,
        http::header::ACCESS_CONTROL_REQUEST_METHOD,
        http::header::ACCESS_CONTROL_REQUEST_HEADERS,
        HeaderName::from_static("solana-client"),
        HeaderName::from_static("sec-fetch-mode"),
    ]);

    let global_middleware = ServiceBuilder::new().layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::DEBUG))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .include_headers(true),
            ),
    );

    let proxy_state = state_arc.clone();
    let proxy_metrics = ServiceBuilder::new().layer(TraceLayer::new_for_http().on_response(
        move |response: &Response, latency: Duration, _span: &Span| {
            proxy_state
                .metrics
                .add_http_call(response.status().into(), "proxy");

            proxy_state.metrics.add_http_latency(
                response.status().into(),
                "proxy",
                latency.as_secs_f64(),
            )
        },
    ));

    let app = Router::new()
        .route("/v1", any(handlers::proxy::handler))
        .route("/v1/", any(handlers::proxy::handler))
        .route("/ws", get(handlers::ws_proxy::handler))
        .route_layer(proxy_metrics)
        .route("/health", get(handlers::health::handler))
        .layer(cors)
        .layer(global_middleware)
        .with_state(state_arc.clone());

    info!("v{}", build_version);
    info!("Running RPC Proxy on port {}", port);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid socket address");

    let private_port = state_arc.config.server.private_port;
    let private_addr = SocketAddr::from(([0, 0, 0, 0], private_port));

    let private_app = Router::new()
        .route("/metrics", get(handlers::metrics::handler))
        .with_state(state_arc.clone());

    let updater = tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            state_arc.update_provider_weights();
        }
    });

    select! {
        _ = shutdown.recv() => info!("Shutdown signal received, killing servers"),
        _ = axum::Server::bind(&private_addr).serve(private_app.into_make_service()) => info!("Private server terminating"),
        _ = axum::Server::bind(&addr).serve(app.into_make_service_with_connect_info::<SocketAddr>()) => info!("Server terminating"),
        _ = updater => info!("Updater terminating")
    }
    Ok(())
}

fn init_providers(config: &Config) -> ProviderRepository {
    // let infura_project_id = config.infura.project_id.clone();
    // let infura_supported_chains = config.infura.supported_chains.clone();
    // let infura_ws_supported_chains = config.infura.supported_ws_chains.clone();
    // let pokt_project_id = config.pokt.project_id.clone();
    // let pokt_supported_chains = config.pokt.supported_chains.clone();

    let mut providers = ProviderRepository::default();
    let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    // TODO: Remove mess

    // let infura_provider = InfuraProvider {
    //     client: forward_proxy_client.clone(),
    //     project_id: infura_project_id.clone(),
    //     supported_chains: infura_supported_chains,
    // };
    // providers.add_provider(Arc::new(infura_provider));
    providers.add_provider(BinanceConfig::default());
    // infura: ,
    //             pokt: ,
    providers.add_provider(PoktConfig::new(
        std::env::var("RPC_PROXY_POKT_PROJECT_ID")
            .expect("Missing RPC_PROXY_POKT_PROJECT_ID env var"),
    ));
    let infura_config = InfuraConfig::new(
        std::env::var("RPC_PROXY_INFURA_PROJECT_ID")
            .expect("Missing RPC_PROXY_INFURA_PROJECT_ID env var"),
    );
    providers.add_provider(infura_config.clone());

    providers.add_ws_provider(infura_config);
    providers.add_provider(OmniatechConfig::default());
    providers.add_provider(ZKSyncConfig::default());
    providers.add_provider(PublicnodeConfig::default());

    // let infura_ws_provider = InfuraWsProvider {
    //     project_id: infura_project_id,
    //     supported_chains: infura_ws_supported_chains,
    // };
    // providers.add_ws_provider(Arc::new(infura_ws_provider));

    // let pokt_provider = PoktProvider {
    //     client: forward_proxy_client.clone(),
    //     project_id: pokt_project_id,
    //     supported_chains: pokt_supported_chains,
    // };
    // providers.add_provider(Arc::new(pokt_provider));

    // let binance_config = BinanceConfig::default();
    // let binance_provider = BinanceProvider {
    //     client: forward_proxy_client.clone(),
    //     supported_chains: binance_config.supported_chains,
    // };
    // providers.add_provider(Arc::new(binance_provider));

    // let zksync_config = ZKSyncConfig::default();
    // let zksync_provider = ZKSyncProvider {
    //     client: forward_proxy_client.clone(),
    //     supported_chains: zksync_config.supported_chains,
    // };
    // providers.add_provider(Arc::new(zksync_provider));

    // let publicnode_config = PublicnodeConfig::default();
    // let publicnode_provider = PublicnodeProvider {
    //     client: forward_proxy_client.clone(),
    //     supported_chains: publicnode_config.supported_chains,
    // };
    // providers.add_provider(Arc::new(publicnode_provider));

    // // Generate and add onerpc provider
    // let onerpc_config = OmniatechConfig::default();
    // let onerpc_provider = OmniatechProvider {
    //     client: forward_proxy_client,
    //     supported_chains: onerpc_config.supported_chains,
    // };
    // providers.add_provider(Arc::new(onerpc_provider));

    providers
}
