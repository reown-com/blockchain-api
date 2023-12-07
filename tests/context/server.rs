use std::net::SocketAddr;

#[cfg(feature = "test-localhost")]
use {
    super::TestResult,
    rpc_proxy::env::{Config, ServerConfig},
    std::net::{Ipv4Addr, SocketAddrV4, TcpStream},
    std::time::Duration,
    std::{env, net::IpAddr},
    tokio::{runtime::Handle, time::sleep},
};

pub struct RpcProxy {
    pub public_addr: String,
    pub public_port: Option<u16>,
    pub private_addr: Option<SocketAddr>,
    pub project_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl RpcProxy {
    #[cfg(feature = "test-localhost")]
    pub async fn start() -> Self {
        let public_port = get_random_port();
        let prometheus_port = get_random_port();
        let hostname = Ipv4Addr::UNSPECIFIED;
        let rt = Handle::current();
        let public_addr = SocketAddr::new(IpAddr::V4(hostname), public_port);
        let private_addr = SocketAddr::new(IpAddr::V4(hostname), prometheus_port);

        let project_id =
            env::var("TEST_RPC_PROXY_PROJECT_ID").expect("TEST_RPC_PROXY_PROJECT_ID must be set");

        std::thread::spawn(move || {
            rt.block_on(async move {
                let mut config: Config = Config::from_env()?;
                config.server = ServerConfig {
                    port: public_port,
                    prometheus_port,
                    host: hostname.to_string(),
                    log_level: "NONE".to_string(),
                    ..Default::default()
                };

                rpc_proxy::bootstrap(config).await
            })
            .unwrap();
        });

        if let Err(e) = wait_for_server_to_start(public_port).await {
            panic!("Failed to start server with error: {e:?}")
        }

        Self {
            public_addr: format!("http://{public_addr}"),
            project_id,
            private_addr: Some(private_addr),
            public_port: Some(public_port),
        }
    }
}

// Finds a free port.
#[cfg(feature = "test-localhost")]
fn get_random_port() -> u16 {
    use std::sync::atomic::{AtomicU16, Ordering};

    static NEXT_PORT: AtomicU16 = AtomicU16::new(9000);

    loop {
        let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);

        if is_port_available(port) {
            return port;
        }
    }
}

#[cfg(feature = "test-localhost")]
fn is_port_available(port: u16) -> bool {
    TcpStream::connect(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).is_err()
}

#[cfg(feature = "test-localhost")]
async fn wait_for_server_to_start(port: u16) -> TestResult<()> {
    let poll_fut = async {
        while is_port_available(port) {
            sleep(Duration::from_millis(10)).await;
        }
    };

    Ok(tokio::time::timeout(Duration::from_secs(5), poll_fut).await?)
}
