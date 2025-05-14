use {
    crate::{
        env::Config,
        handlers::{
            balance::BalanceResponseBody, identity::IdentityResponse, rate_limit_middleware,
            status_latency_metrics_middleware,
        },
        metrics::Metrics,
        project::Registry,
        providers::ProvidersConfig,
        storage::{irn, redis, KeyValueStorage},
    },
    anyhow::Context,
    aws_config::meta::region::RegionProviderChain,
    aws_sdk_s3::{config::Region, Client as S3Client},
    axum::{
        extract::connect_info::IntoMakeServiceWithConnectInfo,
        middleware,
        routing::{get, post},
        Router,
    },
    env::{
        AllnodesConfig, ArbitrumConfig, AuroraConfig, BaseConfig, BinanceConfig, DrpcConfig,
        DuneConfig, EdexaConfig, GetBlockConfig, InfuraConfig, MantleConfig, MonadConfig, MorphConfig,
        NearConfig, OdysseyConfig, PoktConfig, PublicnodeConfig, QuicknodeConfig, SolScanConfig,
        SyndicaConfig, UnichainConfig, WemixConfig, ZKSyncConfig, ZerionConfig, ZoraConfig,
    },
    error::RpcResult,
    http::Request,
    hyper::{header::HeaderName, http, server::conn::AddrIncoming, Body, Server},
    providers::{
        AllnodesProvider, ArbitrumProvider, AuroraProvider, BaseProvider, BinanceProvider,
        DrpcProvider, DuneProvider, EdexaProvider, GetBlockProvider, InfuraProvider, InfuraWsProvider,
        MantleProvider, MonadProvider, MorphProvider, NearProvider, OdysseyProvider, PoktProvider,
        ProviderRepository, PublicnodeProvider, QuicknodeProvider, SolScanProvider,
        SyndicaProvider, UnichainProvider, WemixProvider, ZKSyncProvider, ZerionProvider,
        ZoraProvider, ZoraWsProvider,
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
    tracing::{error, info, log::warn},
    utils::rate_limit::RateLimit,
    wc::{
        geoip::{
            block::{middleware::GeoBlockLayer, BlockingPolicy},
            MaxMindResolver,
        },
        metrics::ServiceMetrics,
    },
};

const DB_STATS_POLLING_INTERVAL: Duration = Duration::from_secs(3600);

mod analytics;
pub mod database;
pub mod env;
pub mod error;
pub mod handlers;
mod json_rpc;
mod metrics;
pub mod names;
pub mod profiler;
mod project;
pub mod providers;
mod state;
mod storage;
pub mod test_helpers;
pub mod utils;
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
                config.rate_limiting.ip_whitelist.clone(),
            ) {
                (Some(max_tokens), Some(refill_interval_sec), Some(refill_rate), ip_whitelist) => {
                    info!(
                        "Rate limiting is enabled with the following configuration: \
                         max_tokens={}, refill_interval_sec={}, refill_rate={}, ip_whitelist={:?}",
                        max_tokens, refill_interval_sec, refill_rate, ip_whitelist
                    );
                    RateLimit::new(
                        redis_addr.write(),
                        config.storage.redis_max_connections,
                        max_tokens,
                        chrono::Duration::seconds(refill_interval_sec as i64),
                        refill_rate,
                        metrics.clone(),
                        ip_whitelist,
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
    let balance_cache = config
        .storage
        .project_data_redis_addr()
        .map(|addr| redis::Redis::new(&addr, config.storage.redis_max_connections))
        .transpose()?
        .map(|r| Arc::new(r) as Arc<dyn KeyValueStorage<BalanceResponseBody> + 'static>);

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
    let irn_client =
        if let (Some(nodes), Some(key_base64), Some(namespace), Some(namespace_secret)) = (
            config.irn.nodes.clone(),
            config.irn.key.clone(),
            config.irn.namespace.clone(),
            config.irn.namespace_secret.clone(),
        ) {
            Some(irn::Irn::new(key_base64, nodes, namespace, namespace_secret).await?)
        } else {
            warn!("IRN client is disabled (missing required environment configuration variables)");
            None
        };

    let state = state::new_state(
        config.clone(),
        postgres.clone(),
        providers,
        metrics.clone(),
        registry,
        analytics,
        http_client,
        rate_limiting,
        irn_client,
        identity_cache,
        balance_cache,
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

    let tracing_layer = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_string();
                tracing::info_span!(
                    "http-request",
                    method = ?request.method(),
                    request_id = ?request_id,
                    uri = request.uri().path()
                )
            }),
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
        .route(
            "/v1/onramp/multi/quotes",
            post(handlers::onramp::multi_quotes::handler),
        )
        .route(
            "/v1/onramp/providers",
            get(handlers::onramp::providers::handler),
        )
        .route(
            "/v1/onramp/providers/properties",
            get(handlers::onramp::properties::handler),
        )
        .route(
            "/v1/onramp/widget",
            post(handlers::onramp::widget::handler),
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
        // Sessions
        .route("/v1/sessions/:address", post(handlers::sessions::create::handler))
        .route("/v1/sessions/:address", get(handlers::sessions::list::handler))
        .route("/v1/sessions/:address/getcontext", get(handlers::sessions::get::handler))
        .route("/v1/sessions/:address/activate", post(handlers::sessions::context::handler))
        .route("/v1/sessions/:address/revoke", post(handlers::sessions::revoke::handler))
        .route("/v1/sessions/:address/sign", post(handlers::sessions::cosign::handler))
        // Bundler
        .route("/v1/bundler", post(handlers::bundler::handler))
        // Wallet
        .route("/v1/wallet", post(handlers::wallet::handler::handler))
        // Same handler as the Wallet 
        .route("/v1/json-rpc", post(handlers::wallet::handler::handler))
        // Chain agnostic orchestration
        .route("/v1/ca/orchestrator/route", post(handlers::chain_agnostic::route::handler_v1))
        .route("/v2/ca/orchestrator/route", post(handlers::chain_agnostic::route::handler_v2))
        .route("/v1/ca/orchestrator/status", get(handlers::chain_agnostic::status::handler))
        // Health
        .route("/health", get(handlers::health::handler))
        .route_layer(tracing_layer)
        .route_layer(cors);

    // Response statuses and latency metrics middleware
    let app = app.layer(middleware::from_fn_with_state(
        state_arc.clone(),
        status_latency_metrics_middleware,
    ));

    // GeoBlock middleware
    let app = if let Some(geoblock) = geoblock {
        app.route_layer(geoblock)
    } else {
        app
    };

    // Rate-limiting middleware
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

    let public_server = create_server(app, &addr);
    let private_server = create_server(private_app, &private_addr);

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
        // Spawning a new task to observe metrics from the database by interval polling
        tokio::spawn({
            let postgres = state_arc.postgres.clone();
            async move {
                let mut interval = tokio::time::interval(DB_STATS_POLLING_INTERVAL);
                loop {
                    interval.tick().await;
                    metrics.update_account_names_count(&postgres).await;
                }
            }
        }),
    ];

    if let Err(e) = futures_util::future::select_all(services).await.0 {
        warn!("Server error: {:?}", e);
    };

    Ok(())
}

fn create_server(
    app: Router,
    addr: &SocketAddr,
) -> Server<AddrIncoming, IntoMakeServiceWithConnectInfo<Router, SocketAddr>> {
    axum::Server::bind(addr).serve(app.into_make_service_with_connect_info::<SocketAddr>())
}

fn init_providers(config: &ProvidersConfig) -> ProviderRepository {
    // Redis pool for providers responses caching where needed
    let mut redis_pool = None;
    if let Some(redis_addr) = &config.cache_redis_addr {
        let redis_builder = deadpool_redis::Config::from_url(redis_addr)
            .builder()
            .map_err(|e| {
                error!(
                    "Failed to create redis pool builder for provider's responses caching: {:?}",
                    e
                );
            })
            .expect("Failed to create redis pool builder for provider's responses caching, builder is None");
        redis_pool = Some(Arc::new(
            redis_builder
                .runtime(deadpool_redis::Runtime::Tokio1)
                .build()
                .expect("Failed to create redis pool"),
        ));
    };

    // Keep in-sync with SUPPORTED_CHAINS.md

    let mut providers = ProviderRepository::new(config);
    providers.add_rpc_provider::<AuroraProvider, AuroraConfig>(AuroraConfig::default());
    providers.add_rpc_provider::<ArbitrumProvider, ArbitrumConfig>(ArbitrumConfig::default());
    providers.add_rpc_provider::<PoktProvider, PoktConfig>(PoktConfig::new(
        config.pokt_project_id.clone(),
    ));

    providers.add_rpc_provider::<BaseProvider, BaseConfig>(BaseConfig::default());
    providers.add_rpc_provider::<BinanceProvider, BinanceConfig>(BinanceConfig::default());
    providers.add_rpc_provider::<ZKSyncProvider, ZKSyncConfig>(ZKSyncConfig::default());
    providers.add_rpc_provider::<PublicnodeProvider, PublicnodeConfig>(PublicnodeConfig::default());
    providers.add_rpc_provider::<QuicknodeProvider, QuicknodeConfig>(QuicknodeConfig::new(
        config.quicknode_api_tokens.clone(),
    ));
    providers.add_rpc_provider::<InfuraProvider, InfuraConfig>(InfuraConfig::new(
        config.infura_project_id.clone(),
    ));
    providers.add_rpc_provider::<ZoraProvider, ZoraConfig>(ZoraConfig::default());
    providers.add_rpc_provider::<NearProvider, NearConfig>(NearConfig::default());
    providers.add_rpc_provider::<MantleProvider, MantleConfig>(MantleConfig::default());
    providers.add_rpc_provider::<UnichainProvider, UnichainConfig>(UnichainConfig::default());
    providers.add_rpc_provider::<SyndicaProvider, SyndicaConfig>(SyndicaConfig::new(
        config.syndica_api_key.clone(),
    ));
    providers.add_rpc_provider::<MorphProvider, MorphConfig>(MorphConfig::default());
    providers.add_rpc_provider::<WemixProvider, WemixConfig>(WemixConfig::default());
    providers.add_rpc_provider::<DrpcProvider, DrpcConfig>(DrpcConfig::default());
    providers.add_rpc_provider::<OdysseyProvider, OdysseyConfig>(OdysseyConfig::default());
    providers.add_rpc_provider::<EdexaProvider, EdexaConfig>(EdexaConfig::default());
    providers.add_rpc_provider::<AllnodesProvider, AllnodesConfig>(AllnodesConfig::new(
        config.allnodes_api_key.clone(),
    ));
    providers.add_rpc_provider::<MonadProvider, MonadConfig>(MonadConfig::default());

    if let Some(getblock_access_tokens) = &config.getblock_access_tokens {
        providers.add_rpc_provider::<GetBlockProvider, GetBlockConfig>(GetBlockConfig::new(
            getblock_access_tokens.clone(),
        ));
    };

    providers.add_ws_provider::<InfuraWsProvider, InfuraConfig>(InfuraConfig::new(
        config.infura_project_id.clone(),
    ));
    providers.add_ws_provider::<ZoraWsProvider, ZoraConfig>(ZoraConfig::default());

    providers.add_balance_provider::<ZerionProvider, ZerionConfig>(
        ZerionConfig::new(config.zerion_api_key.clone()),
        None,
    );
    providers.add_balance_provider::<DuneProvider, DuneConfig>(
        DuneConfig::new(config.dune_api_key.clone()),
        None,
    );
    providers.add_balance_provider::<SolScanProvider, SolScanConfig>(
        SolScanConfig::new(config.solscan_api_v2_token.clone()),
        redis_pool.clone(),
    );

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
