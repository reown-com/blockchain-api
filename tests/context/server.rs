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
    pub public_addr: String,
    pub public_port: Option<u16>,
    pub private_addr: Option<SocketAddr>,
    pub project_id: String,
    pub shutdown_signal: Option<tokio::sync::broadcast::Sender<()>>,
    pub is_shutdown: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl RpcProxy {
    #[allow(unused)]
    pub async fn start() -> Self {
        let (public_port, private_port) = get_random_ports();
        let hostname = Ipv4Addr::UNSPECIFIED;
        let rt = Handle::current();
        let public_addr = SocketAddr::new(IpAddr::V4(hostname), public_port);
        let private_addr = SocketAddr::new(IpAddr::V4(hostname), private_port);

        let (signal, shutdown) = broadcast::channel(1);

        let project_id =
            env::var("TEST_RPC_PROXY_PROJECT_ID").expect("TEST_RPC_PROXY_PROJECT_ID must be set");

        std::thread::spawn(move || {
            rt.block_on(async move {
                let mut config: Config = Config::from_env()?;
                config.server = ServerConfig {
                    port: public_port,
                    private_port,
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
            public_addr: format!("http://{}", public_addr),
            project_id,
            shutdown_signal: Some(signal),
            is_shutdown: false,
            private_addr: Some(private_addr),
            public_port: Some(public_port),
        }
    }

    #[allow(unused)]
    pub async fn shutdown(&mut self) {
        if self.is_shutdown {
            return;
        }
        self.is_shutdown = true;
        let sender = self.shutdown_signal.clone();
        let _ = sender.unwrap().send(());
        wait_for_server_to_shutdown(self.public_port.unwrap())
            .await
            .unwrap();
    }
}

// Finds a free port.
fn get_random_ports() -> (u16, u16) {
    use std::sync::atomic::{AtomicU16, Ordering};

    static NEXT_PORT: AtomicU16 = AtomicU16::new(9000);

    loop {
        let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);

        if is_port_available(port) {
            let pub_port = port;
            loop {
                let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);
                if is_port_available(port) {
                    return (pub_port, port);
                }
            }
        }
    }
}

fn is_port_available(port: u16) -> bool {
    TcpStream::connect(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).is_err()
}

#[allow(unused)]
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
