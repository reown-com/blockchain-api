use {
    dotenv::dotenv,
    rpc_proxy::{env::Config, error},
    std::str::FromStr,
    tokio::sync::broadcast,
    tracing_subscriber::fmt::format::FmtSpan,
};

#[tokio::main]
async fn main() -> error::RpcResult<()> {
    dotenv().ok();

    let (_signal, shutdown) = broadcast::channel(1);

    let config = Config::from_env()
        .map_err(|e| dbg!(e))
        .expect("Failed to load config, please ensure all env vars are defined.");

    tracing_subscriber::fmt()
        .with_max_level(
            tracing::Level::from_str(config.server.log_level.as_str()).expect("Invalid log level"),
        )
        .with_span_events(FmtSpan::CLOSE)
        .with_ansi(false)
        .init();

    rpc_proxy::bootstrap(shutdown, config).await
}
