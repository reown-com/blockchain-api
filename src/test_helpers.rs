use crate::env::{Config, ServerConfig};
use crate::providers::mock_alto::MockAltoUrls;
use std::sync::atomic::{AtomicU16, Ordering};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
    time::Duration,
};
use tokio::runtime::Handle;
use url::Url;

pub struct Params {
    pub validate_project_id: bool,
    pub override_bundler_urls: Option<MockAltoUrls>,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            validate_project_id: true,
            override_bundler_urls: None,
        }
    }
}

pub async fn spawn_blockchain_api() -> Url {
    spawn_blockchain_api_with_params(Default::default()).await
}

pub async fn spawn_blockchain_api_with_params(params: Params) -> Url {
    // dotenv::dotenv().ok();

    let public_port = get_random_port();
    let prometheus_port = get_random_port();
    let hostname = Ipv4Addr::UNSPECIFIED;
    let rt = Handle::current();
    let public_addr = SocketAddr::new(IpAddr::V4(hostname), public_port);

    println!("RPC_PROXY_POSTGRES_URI 1: {}", std::env::var("RPC_PROXY_POSTGRES_URI").unwrap());
    std::thread::spawn(move || {
        println!("RPC_PROXY_POSTGRES_URI 2: {}", std::env::var("RPC_PROXY_POSTGRES_URI").unwrap());
        rt.block_on(async move {
            println!("RPC_PROXY_POSTGRES_URI 3: {}", std::env::var("RPC_PROXY_POSTGRES_URI").unwrap());
            let mut config = Config::from_env()?;
            config.server = ServerConfig {
                port: public_port,
                prometheus_port,
                host: hostname.to_string(),
                log_level: "NONE".to_string(),
                validate_project_id: params.validate_project_id,
                ..Default::default()
            };
            config.providers.override_bundler_urls = params.override_bundler_urls;

            crate::bootstrap(config).await
        })
        .unwrap();
    });

    wait_for_server_to_start(public_port).await;

    format!("http://{public_addr}").parse().unwrap()
}

fn get_random_port() -> u16 {
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

async fn wait_for_server_to_start(port: u16) {
    let poll_fut = async {
        while is_port_available(port) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    };

    tokio::time::timeout(Duration::from_secs(5), poll_fut)
        .await
        .unwrap()
}
