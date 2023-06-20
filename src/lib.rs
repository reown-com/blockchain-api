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
    hyper::header::HeaderName,
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
    tower::ServiceBuilder,
    tower_http::{
        cors::{Any, CorsLayer},
        trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    },
    tracing::{info, log::warn, Level, Span},
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

pub async fn bootstrap(config: Config) -> RpcResult<()> {
    let prometheus_exporter = opentelemetry_prometheus::exporter().init();
    let meter = prometheus_exporter
        .provider()
        .unwrap()
        .meter("rpc-proxy", None);

    let metrics = Arc::new(Metrics::new(&meter));
    let registry = Registry::new(&config.registry, &config.storage, &meter)?;
    let providers = init_providers();

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
        .route("/v1/identity/:address", get(handlers::identity::handler))
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

    let public_server =
        axum::Server::bind(&addr).serve(app.into_make_service_with_connect_info::<SocketAddr>());

    let private_server = axum::Server::bind(&private_addr)
        .serve(private_app.into_make_service_with_connect_info::<SocketAddr>());

    let updater = async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            state_arc.update_provider_weights().await;
        }
    };

    let services = vec![
        tokio::spawn(public_server),
        tokio::spawn(private_server),
        tokio::spawn(updater),
    ];

    if let Err(e) = futures_util::future::select_all(services).await.0 {
        warn!("Server error: {:?}", e);
    };

    Ok(())
}

fn init_providers() -> ProviderRepository {
    let mut providers = ProviderRepository::new();

    let infura_project_id = std::env::var("RPC_PROXY_INFURA_PROJECT_ID")
        .expect("Missing RPC_PROXY_INFURA_PROJECT_ID env var");

    providers.add_provider::<PoktProvider, PoktConfig>(PoktConfig::new(
        std::env::var("RPC_PROXY_POKT_PROJECT_ID")
            .expect("Missing RPC_PROXY_POKT_PROJECT_ID env var"),
    ));

    providers.add_provider::<BinanceProvider, BinanceConfig>(BinanceConfig::default());
    providers.add_provider::<OmniatechProvider, OmniatechConfig>(OmniatechConfig::default());
    providers.add_provider::<ZKSyncProvider, ZKSyncConfig>(ZKSyncConfig::default());
    providers.add_provider::<PublicnodeProvider, PublicnodeConfig>(PublicnodeConfig::default());
    providers
        .add_provider::<InfuraProvider, InfuraConfig>(InfuraConfig::new(infura_project_id.clone()));

    providers
        .add_ws_provider::<InfuraWsProvider, InfuraConfig>(InfuraConfig::new(infura_project_id));

    providers
}
