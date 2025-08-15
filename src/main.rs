use {
    dotenv::dotenv,
    rpc_proxy::{env::Config, error},
    tracing::level_filters::LevelFilter,
    tracing_subscriber::{fmt::format::FmtSpan, EnvFilter},
};

#[global_allocator]
static ALLOC: wc::alloc::Jemalloc = wc::alloc::Jemalloc;

#[tokio::main]
async fn main() -> error::RpcResult<()> {
    dotenv().ok();

    // Install default CryptoProvider for rustls (required since rustls 0.23)
    // Prefer ring backend by default.
    rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider())
        .expect("failed to install default rustls CryptoProvider");

    let config = Config::from_env()
        .map_err(|e| dbg!(e))
        .expect("Failed to load config, please ensure all env variables are defined.");

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::ERROR.into())
                .parse(&config.server.log_level)
                .expect("Invalid log level"),
        )
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(false)
        .init();

    rpc_proxy::bootstrap(config).await
}
