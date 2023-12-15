use {
    dotenv::dotenv,
    rpc_proxy::{env::Config, error},
    std::str::FromStr,
    tracing_subscriber::fmt::format::FmtSpan,
};

#[global_allocator]
static ALLOC: wc::alloc::Jemalloc = wc::alloc::Jemalloc;

#[tokio::main]
async fn main() -> error::RpcResult<()> {
    dotenv().ok();

    let config = Config::from_env()
        .map_err(|e| dbg!(e))
        .expect("Failed to load config, please ensure all env variables are defined.");

    tracing_subscriber::fmt()
        .with_max_level(
            tracing::Level::from_str(config.server.log_level.as_str()).expect("Invalid log level"),
        )
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(false)
        .init();

    rpc_proxy::bootstrap(config).await
}
