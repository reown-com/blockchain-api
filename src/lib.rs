use {
    crate::{
        env::Config,
        handlers::{identity::IdentityResponse, rate_limit_middleware},
        metrics::Metrics,
        project::Registry,
        providers::ProvidersConfig,
        storage::{redis, KeyValueStorage},
    },
    anyhow::Context,
    aws_config::meta::region::RegionProviderChain,
    aws_sdk_s3::{config::Region, Client as S3Client},
    axum::{
        extract::connect_info::IntoMakeServiceWithConnectInfo,
        middleware,
        response::Response,
        routing::{get, post},
        Router,
    },
    env::{
        AuroraConfig,
        BaseConfig,
        BinanceConfig,
        GetBlockConfig,
        InfuraConfig,
        MantleConfig,
        NearConfig,
        PoktConfig,
        PublicnodeConfig,
        QuicknodeConfig,
        ZKSyncConfig,
        ZoraConfig,
    },
    error::RpcResult,
    http::Request,
    hyper::{header::HeaderName, http, server::conn::AddrIncoming, Body, Server},
    providers::{
        AuroraProvider,
        BaseProvider,
        BinanceProvider,
        GetBlockProvider,
        InfuraProvider,
        InfuraWsProvider,
        MantleProvider,
        NearProvider,
        PoktProvider,
        ProviderRepository,
        PublicnodeProvider,
        QuicknodeProvider,
        ZKSyncProvider,
        ZoraProvider,
        ZoraWsProvider,
    },
    sqlx::postgres::PgPoolOptions,
    std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
        time::Duration,
    },
    tower::ServiceBuilder,
    tower_http::{
        cors::{Any, CorsLayer},
        request_id::MakeRequestUuid,
        trace::TraceLayer,
        ServiceBuilderExt,
    },
    tracing::{info, log::warn, Span},
    utils::rate_limit::RateLimit,
    wc::{
        geoip::{
            block::{middleware::GeoBlockLayer, BlockingPolicy},
            MaxMindResolver,
        },
        http::ServiceTaskExecutor,
        metrics::ServiceMetrics,
    },
};

const SERVICE_TASK_TIMEOUT: Duration = Duration::from_secs(15);
const KEEPALIVE_IDLE_DURATION: Duration = Duration::from_secs(60);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(10);
const KEEPALIVE_RETRIES: u32 = 5;

mod analytics;
pub mod database;
pub mod env;
pub mod error;
mod extractors;
pub mod handlers;
mod json_rpc;
mod metrics;
pub mod profiler;
mod project;
pub mod providers;
mod state;
mod storage;
mod utils;
mod ws;

pub async fn bootstrap(config: Config) -> RpcResult<()> {
    ServiceMetrics::init_with_name("rpc-proxy");

    let s3_client = get_s3_client(&config).await;
    let geoip_resolver = get_geoip_resolver(&config, &s3_client).await;

    let metrics = Arc::new(Metrics::new());
    let registry = Registry::new(&config.registry, &config.storage)?;

    // Rate limiting construction
    let rate_limiting = match config.storage.rate_limiting_cache_redis_addr() {
        None => {
            warn!("Rate limiting is disabled (no redis caching endpoint provided)");
            None
        }
        Some(redis_addr) => {
            match (
                config.rate_limiting.max_tokens,
                config.rate_limiting.refill_interval_sec,
                config.rate_limiting.refill_rate,
            ) {
                (Some(max_tokens), Some(refill_interval_sec), Some(refill_rate)) => {
                    info!(
                        "Rate limiting is enabled with the following configuration: \
                         max_tokens={}, refill_interval_sec={}, refill_rate={}",
                        max_tokens, refill_interval_sec, refill_rate
                    );
                    RateLimit::new(
                        redis_addr.write(),
                        max_tokens,
                        chrono::Duration::seconds(refill_interval_sec as i64),
                        refill_rate,
                    )
                }
                _ => {
                    warn!("Rate limiting is disabled (missing env configuration variables)");
                    None
                }
            }
        }
    };

    // TODO refactor encapsulate these details in a lower layer
    let identity_cache = config
        .storage
        .project_data_redis_addr()
        .map(|addr| redis::Redis::new(&addr, config.storage.redis_max_connections))
        .transpose()?
        .map(|r| Arc::new(r) as Arc<dyn KeyValueStorage<IdentityResponse> + 'static>);

    let providers = init_providers(&config.providers);

    let external_ip = config
        .server
        .external_ip()
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

    let analytics = analytics::RPCAnalytics::new(
        &config.analytics,
        s3_client,
        geoip_resolver.clone(),
        external_ip,
    )
    .await
    .context("failed to init analytics")?;

    let geoblock = analytics.geoip_resolver().as_ref().map(|resolver| {
        // let r = resolver.clone().deref();
        GeoBlockLayer::new(
            resolver.clone(),
            config.server.blocked_countries.clone(),
            BlockingPolicy::AllowAll,
        )
    });

    let postgres = PgPoolOptions::new()
        .max_connections(config.postgres.max_connections.into())
        .connect(&config.postgres.uri)
        .await?;
    sqlx::migrate!("./migrations").run(&postgres).await?;

    let http_client = reqwest::Client::new();

    let state = state::new_state(
        config.clone(),
        postgres.clone(),
        providers,
        metrics.clone(),
        registry,
        identity_cache,
        analytics,
        http_client,
        rate_limiting,
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
        HeaderName::from_static("x-sdk-type"),
        HeaderName::from_static("x-sdk-version"),
    ]);

    let proxy_state = state_arc.clone();
    let tracing_and_metrics_layer = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let request_id = match request.headers().get("x-request-id") {
                    Some(value) => value.to_str().unwrap_or_default().to_string(),
                    None => {
                        // If this warning is triggered, it means that the `x-request-id` was not
                        // propagated to headers properly. This is a bug in the middleware chain.
                        warn!("Missing x-request-id header in a middleware");
                        String::new()
                    }
                };
                tracing::info_span!("http-request", "method" = ?request.method(), "request_id" = ?request_id, "uri" = ?request.uri())
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
            ),
        )
        .propagate_x_request_id();

    let app = Router::new()
        .route("/v1", post(handlers::proxy::handler))
        .route("/v1/", post(handlers::proxy::handler))
        .route("/v1/supported-chains", get(handlers::supported_chains::handler))
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
        .route(
            "/v1/account/:address/portfolio",
            get(handlers::portfolio::handler),
        )
        .route(
            "/v1/account/:address/balance",
            get(handlers::balance::handler),
        )
        // Register account name
        .route(
            "/v1/profile/account",
            post(handlers::profile::register::handler),
        )
         // Update account name attributes
         .route(
            "/v1/profile/account/:name/attributes",
            post(handlers::profile::attributes::handler),
        )
        // Update account name address
        .route(
            "/v1/profile/account/:name/address",
            post(handlers::profile::address::handler),
        )
        // Forward address lookup
        .route(
            "/v1/profile/account/:name",
            get(handlers::profile::lookup::handler),
        )
        // Reverse name lookup
        .route(
            "/v1/profile/reverse/:address",
            get(handlers::profile::reverse::handler),
        )
        // Reverse name lookup
        .route(
            "/v1/profile/suggestions/:name",
            get(handlers::profile::suggestions::handler),
        )
        // Generators
        .route(
            "/v1/generators/onrampurl",
            post(handlers::generators::onrampurl::handler),
        )
        // OnRamp
        .route(
            "/v1/onramp/buy/options",
            get(handlers::onramp::options::handler),
        )
        .route(
            "/v1/onramp/buy/quotes",
            get(handlers::onramp::quotes::handler),
        )
        // Conversion
        .route(
            "/v1/convert/tokens",
            get(handlers::convert::tokens::handler),
        )
        .route(
            "/v1/convert/quotes",
            get(handlers::convert::quotes::handler),
        )
        .route(
            "/v1/convert/build-approve",
            get(handlers::convert::approve::handler),
        )
        .route(
            "/v1/convert/build-transaction",
            post(handlers::convert::transaction::handler),
        )
        .route(
            "/v1/convert/gas-price",
            get(handlers::convert::gas_price::handler),
        )
        .route(
            "/v1/convert/allowance",
            get(handlers::convert::allowance::handler),
        )
        // Fungible price
        .route(
            "/v1/fungible/price",
            post(handlers::fungible_price::handler),
        )
        .route("/health", get(handlers::health::handler))
        .route_layer(tracing_and_metrics_layer)
        .layer(cors);

    let app = if let Some(geoblock) = geoblock {
        app.layer(geoblock)
    } else {
        app
    };
    let app = if state_arc.rate_limit.is_some() {
        app.route_layer(middleware::from_fn_with_state(
            state_arc.clone(),
            rate_limit_middleware,
        ))
    } else {
        app
    };

    let app = app.with_state(state_arc.clone());

    info!("v{}", build_version);
    info!("Running Blockchain-API server on port {}", port);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid socket address");

    let private_port = state_arc.config.server.prometheus_port;
    let private_addr = SocketAddr::from(([0, 0, 0, 0], private_port));

    info!("Starting metric server on {}", private_addr);

    let private_app = Router::new()
        .route("/metrics", get(handlers::metrics::handler))
        .with_state(state_arc.clone());

    let public_server = create_server("public_server", app, &addr);
    let private_server = create_server("private_server", private_app, &private_addr);

    let weights_updater = {
        let state_arc = state_arc.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            loop {
                interval.tick().await;
                state_arc.clone().update_provider_weights().await;
            }
        }
    };

    let system_metrics_updater = {
        let state_arc = state_arc.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                // Gather system metrics (CPU and Memory usage)
                state_arc.clone().metrics.gather_system_metrics().await;
                // Gather current rate limited in-memory entries count
                if let Some(rate_limit) = &state_arc.rate_limit {
                    state_arc
                        .metrics
                        .add_rate_limited_entries_count(rate_limit.get_rate_limited_count().await);
                }
            }
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
        tokio::spawn(weights_updater),
        tokio::spawn(system_metrics_updater),
        tokio::spawn(profiler),
    ];

    if let Err(e) = futures_util::future::select_all(services).await.0 {
        warn!("Server error: {:?}", e);
    };

    Ok(())
}

fn create_server(
    name: &'static str,
    app: Router,
    addr: &SocketAddr,
) -> Server<AddrIncoming, IntoMakeServiceWithConnectInfo<Router, SocketAddr>, ServiceTaskExecutor> {
    let executor = ServiceTaskExecutor::new()
        .name(Some(name))
        .timeout(Some(SERVICE_TASK_TIMEOUT));

    axum::Server::bind(addr)
        .executor(executor)
        .tcp_keepalive(Some(KEEPALIVE_IDLE_DURATION))
        .tcp_keepalive_interval(Some(KEEPALIVE_INTERVAL))
        .tcp_keepalive_retries(Some(KEEPALIVE_RETRIES))
        .tcp_sleep_on_accept_errors(true)
        .tcp_nodelay(true)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
}

fn init_providers(config: &ProvidersConfig) -> ProviderRepository {
    let mut providers = ProviderRepository::new(config);

    // Keep in-sync with SUPPORTED_CHAINS.md

    providers.add_provider::<AuroraProvider, AuroraConfig>(AuroraConfig::default());
    providers
        .add_provider::<PoktProvider, PoktConfig>(PoktConfig::new(config.pokt_project_id.clone()));

    providers.add_provider::<BaseProvider, BaseConfig>(BaseConfig::default());
    providers.add_provider::<BinanceProvider, BinanceConfig>(BinanceConfig::default());
    providers.add_provider::<ZKSyncProvider, ZKSyncConfig>(ZKSyncConfig::default());
    providers.add_provider::<PublicnodeProvider, PublicnodeConfig>(PublicnodeConfig::default());
    providers.add_provider::<QuicknodeProvider, QuicknodeConfig>(QuicknodeConfig::new(
        config.quicknode_api_token.clone(),
    ));
    providers.add_provider::<InfuraProvider, InfuraConfig>(InfuraConfig::new(
        config.infura_project_id.clone(),
    ));
    providers.add_provider::<ZoraProvider, ZoraConfig>(ZoraConfig::default());
    providers.add_provider::<NearProvider, NearConfig>(NearConfig::default());
    providers.add_provider::<MantleProvider, MantleConfig>(MantleConfig::default());

    if let Some(getblock_access_tokens) = &config.getblock_access_tokens {
        providers.add_provider::<GetBlockProvider, GetBlockConfig>(GetBlockConfig::new(
            getblock_access_tokens.clone(),
        ));
    };

    providers.add_ws_provider::<InfuraWsProvider, InfuraConfig>(InfuraConfig::new(
        config.infura_project_id.clone(),
    ));
    providers.add_ws_provider::<ZoraWsProvider, ZoraConfig>(ZoraConfig::default());

    providers
}

async fn get_s3_client(config: &Config) -> S3Client {
    let region_provider = RegionProviderChain::first_try(Region::new("eu-central-1"));
    let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let aws_config = if let Some(s3_endpoint) = &config.server.s3_endpoint {
        info!(%s3_endpoint, "initializing custom s3 endpoint");

        aws_sdk_s3::config::Builder::from(&shared_config)
            .endpoint_url(s3_endpoint)
            .build()
    } else {
        aws_sdk_s3::config::Builder::from(&shared_config).build()
    };

    S3Client::from_conf(aws_config)
}

async fn get_geoip_resolver(config: &Config, s3_client: &S3Client) -> Option<Arc<MaxMindResolver>> {
    if let (Some(bucket), Some(key)) = (&config.server.geoip_db_bucket, &config.server.geoip_db_key)
    {
        info!(%bucket, %key, "initializing geoip database from aws s3");

        Some(Arc::new(
            MaxMindResolver::from_aws_s3(s3_client, bucket, key)
                .await
                .expect("failed to load geoip resolver"),
        ))
    } else {
        info!("geoip lookup is disabled");
        None
    }
}
