use {
    super::TestResult,
    rpc_proxy::env::{Config, ServerConfig},
    std::{
        env,
        net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
    },
    tokio::{
        runtime::Handle,
        sync::broadcast,
        time::{sleep, Duration},
    },
};

pub struct RpcProxy {
    pub public_addr: SocketAddr,
    pub project_id: String,
    shutdown_signal: tokio::sync::broadcast::Sender<()>,
    is_shutdown: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl RpcProxy {
    pub async fn start() -> Self {
        let public_port = get_random_port();
        let hostname = Ipv4Addr::UNSPECIFIED;
        let rt = Handle::current();
        let public_addr = SocketAddr::new(IpAddr::V4(hostname), public_port);

        let (signal, shutdown) = broadcast::channel(1);

        let project_id =
            env::var("TEST_RPC_PROXY_PROJECT_ID").expect("TEST_RPC_PROXY_PROJECT_ID must be set");

        std::thread::spawn(move || {
            rt.block_on(async move {
                let mut config: Config = Config::from_env()?;
                config.server = ServerConfig {
                    port: public_port,
                    host: hostname.to_string(),
                    log_level: "NONE".to_string(),
                    ..Default::default()
                };

                rpc_proxy::bootstrap(shutdown, config).await
            })
            .unwrap();
        });

        if let Err(e) = wait_for_server_to_start(public_port).await {
            panic!("Failed to start server with error: {e:?}")
        }

        Self {
            public_addr,
            project_id,
            shutdown_signal: signal,
            is_shutdown: false,
        }
    }

    pub async fn shutdown(&mut self) {
        if self.is_shutdown {
            return;
        }
        self.is_shutdown = true;
        let _ = self.shutdown_signal.send(());
        wait_for_server_to_shutdown(self.public_addr.port())
            .await
            .unwrap();
    }
}

// Finds a free port.
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

fn is_port_available(port: u16) -> bool {
    TcpStream::connect(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).is_err()
}

async fn wait_for_server_to_shutdown(port: u16) -> TestResult<()> {
    let poll_fut = async {
        while !is_port_available(port) {
            sleep(Duration::from_millis(10)).await;
        }
    };

    Ok(tokio::time::timeout(Duration::from_secs(3), poll_fut).await?)
}

async fn wait_for_server_to_start(port: u16) -> TestResult<()> {
    let poll_fut = async {
        while is_port_available(port) {
            sleep(Duration::from_millis(10)).await;
        }
    };

    Ok(tokio::time::timeout(Duration::from_secs(5), poll_fut).await?)
}
