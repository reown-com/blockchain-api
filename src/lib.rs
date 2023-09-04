use {
    crate::{
        env::Config,
        handlers::identity::IdentityResponse,
        metrics::Metrics,
        project::Registry,
        storage::{redis, KeyValueStorage},
    },
    anyhow::Context,
    axum::{
        response::Response,
        routing::{get, post},
        Router,
    },
    env::{
        BaseConfig,
        BinanceConfig,
        InfuraConfig,
        OmniatechConfig,
        PoktConfig,
        PublicnodeConfig,
        ZKSyncConfig,
        ZoraConfig,
    },
    error::RpcResult,
    http::Request,
    hyper::{header::HeaderName, http, Body},
    providers::{
        BaseProvider,
        BinanceProvider,
        InfuraProvider,
        InfuraWsProvider,
        OmniatechProvider,
        PoktProvider,
        ProviderRepository,
        PublicnodeProvider,
        ZKSyncProvider,
        ZoraProvider,
        ZoraWsProvider,
    },
    std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
        time::Duration,
    },
    tower::ServiceBuilder,
    tower_http::{
        cors::{Any, CorsLayer},
        trace::TraceLayer,
    },
    tracing::{info, log::warn, Span},
    wc::{http::ServiceTaskExecutor, metrics::ServiceMetrics},
};

const SERVICE_TASK_TIMEOUT: Duration = Duration::from_secs(15);
const KEEPALIVE_IDLE_DURATION: Duration = Duration::from_secs(5);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(5);
const KEEPALIVE_RETRIES: u32 = 1;

mod analytics;
pub mod env;
pub mod error;
mod extractors;
mod handlers;
mod json_rpc;
mod metrics;
pub mod profiler;
mod project;
mod providers;
mod state;
mod storage;
mod utils;
mod ws;

pub async fn bootstrap(config: Config) -> RpcResult<()> {
    ServiceMetrics::init_with_name("rpc-proxy");

    let metrics = Arc::new(Metrics::new());
    let registry = Registry::new(&config.registry, &config.storage)?;
    // TODO refactor encapsulate these details in a lower layer
    let identity_cache = config
        .storage
        .project_data_redis_addr()
        .map(|addr| redis::Redis::new(&addr, config.storage.redis_max_connections))
        .transpose()?
        .map(|r| Arc::new(r) as Arc<dyn KeyValueStorage<IdentityResponse> + 'static>);
    let providers = init_providers();

    let external_ip = config
        .server
        .external_ip()
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

    let analytics = analytics::RPCAnalytics::new(&config.analytics, external_ip)
        .await
        .context("failed to init analytics")?;

    let state = state::new_state(
        config.clone(),
        providers,
        metrics.clone(),
        registry,
        identity_cache,
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

    let proxy_state = state_arc.clone();
    let proxy_metrics = ServiceBuilder::new().layer(TraceLayer::new_for_http()
    .make_span_with(|request: &Request<Body>| {
        tracing::info_span!("http-request", "method" = ?request.method(), "uri" = ?request.uri())
    })
    .on_response(
        move |response: &Response, latency: Duration, _span: &Span| {
            proxy_state
                .metrics
                .add_http_call(response.status().into(), "proxy".to_owned());

            proxy_state.metrics.add_http_latency(
                response.status().into(),
                "proxy".to_owned(),
                latency.as_secs_f64(),
            )
        },
    )
    );

    let app = Router::new()
        .route("/v1", post(handlers::proxy::handler))
        .route("/v1/", post(handlers::proxy::handler))
        .route("/ws", get(handlers::ws_proxy::handler))
        .route("/v1/identity/:address", get(handlers::identity::handler))
        .route(
            "/v1/account/:address/identity",
            get(handlers::identity::handler),
        )
        .route(
            "/v1/account/:address/history",
            get(handlers::history::handler),
        )
        .route_layer(proxy_metrics)
        .route("/health", get(handlers::health::handler))
        .layer(cors)
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

    let executor = ServiceTaskExecutor::new()
        .name(Some("public_server"))
        .timeout(Some(SERVICE_TASK_TIMEOUT));

    let public_server = axum::Server::bind(&addr)
        .executor(executor)
        .tcp_keepalive(Some(KEEPALIVE_IDLE_DURATION))
        .tcp_keepalive_interval(Some(KEEPALIVE_INTERVAL))
        .tcp_keepalive_retries(Some(KEEPALIVE_RETRIES))
        .tcp_sleep_on_accept_errors(false)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>());

    let executor = ServiceTaskExecutor::new()
        .name(Some("private_server"))
        .timeout(Some(SERVICE_TASK_TIMEOUT));

    let private_server = axum::Server::bind(&private_addr)
        .executor(executor)
        .tcp_keepalive(Some(KEEPALIVE_IDLE_DURATION))
        .tcp_keepalive_interval(Some(KEEPALIVE_INTERVAL))
        .tcp_keepalive_retries(Some(KEEPALIVE_RETRIES))
        .tcp_sleep_on_accept_errors(false)
        .serve(private_app.into_make_service_with_connect_info::<SocketAddr>());

    let updater = async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            state_arc.update_provider_weights().await;
        }
    };

    let profiler = async move {
        if let Err(e) = tokio::spawn(profiler::run()).await {
            warn!("Memory debug stats collection failed with: {:?}", e);
        }
        Ok(())
    };

    let services = vec![
        tokio::spawn(public_server),
        tokio::spawn(private_server),
        tokio::spawn(updater),
        tokio::spawn(profiler),
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

    // Keep in-sync with SUPPORTED_CHAINS.md

    providers.add_provider::<PoktProvider, PoktConfig>(PoktConfig::new(
        std::env::var("RPC_PROXY_POKT_PROJECT_ID")
            .expect("Missing RPC_PROXY_POKT_PROJECT_ID env var"),
    ));

    providers.add_provider::<BaseProvider, BaseConfig>(BaseConfig::default());
    providers.add_provider::<BinanceProvider, BinanceConfig>(BinanceConfig::default());
    providers.add_provider::<OmniatechProvider, OmniatechConfig>(OmniatechConfig::default());
    providers.add_provider::<ZKSyncProvider, ZKSyncConfig>(ZKSyncConfig::default());
    providers.add_provider::<PublicnodeProvider, PublicnodeConfig>(PublicnodeConfig::default());
    providers
        .add_provider::<InfuraProvider, InfuraConfig>(InfuraConfig::new(infura_project_id.clone()));
    providers.add_provider::<ZoraProvider, ZoraConfig>(ZoraConfig::default());

    providers
        .add_ws_provider::<InfuraWsProvider, InfuraConfig>(InfuraConfig::new(infura_project_id));
    providers.add_ws_provider::<ZoraWsProvider, ZoraConfig>(ZoraConfig::default());

    providers
}
