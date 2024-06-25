use {
    super::StorageError,
    irn_api::{auth::Auth, Client},
    serde::Deserialize,
    std::net::SocketAddr,
    std::time::Duration,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_OPERATION_TIME: Duration = Duration::from_millis(2500);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);
const UDP_SOCKET_COUNT: usize = 1;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub node: Option<String>,
    pub key: Option<String>,
    pub namespace: Option<String>,
    pub namespace_secret: Option<String>,
}

#[derive(Clone)]
pub struct Irn {
    #[allow(dead_code)]
    client: Client,
}

impl Irn {
    pub fn new(
        node_addr: String,
        key_base64: String,
        namespace: String,
        namespace_secret: String,
    ) -> Result<Self, StorageError> {
        let key = irn_api::auth::client_key_from_bytes(
            key_base64.as_bytes(),
            irn_api::auth::Encoding::Base64,
        )
        .map_err(|_| StorageError::WrongKey(key_base64))?;
        let peer_id = irn_api::auth::peer_id(&key.verifying_key());
        let node_addr = node_addr
            .parse::<SocketAddr>()
            .map_err(|_| StorageError::WrongNodeAddress(node_addr))?;
        let address = (peer_id, irn_network::socketaddr_to_multiaddr(node_addr));
        let namespaces = vec![
            Auth::from_secret(namespace_secret.as_bytes(), namespace.as_bytes())
                .map_err(|_| StorageError::WrongNamespace(namespace))?,
        ];

        let client = Client::new(irn_api::client::Config {
            key,
            nodes: [address].into(),
            shadowing_nodes: Default::default(),
            shadowing_factor: 0.0,
            request_timeout: REQUEST_TIMEOUT,
            max_operation_time: MAX_OPERATION_TIME,
            connection_timeout: CONNECTION_TIMEOUT,
            udp_socket_count: UDP_SOCKET_COUNT,
            namespaces,
        })?;

        Ok(Self { client })
    }
}
