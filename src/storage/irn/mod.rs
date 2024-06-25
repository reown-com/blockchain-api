use {
    super::StorageError,
    irn_api::{
        auth::{Auth, PublicKey},
        Client, Key,
    },
    serde::Deserialize,
    std::net::SocketAddr,
    std::time::Duration,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_OPERATION_TIME: Duration = Duration::from_secs(3);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);
const RECORDS_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 90); // 90 days
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
    client: Client,
    namespace: PublicKey,
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
        // IRN connection error Network(ConnectionHandler(NoAvailablePeers)) when using
        // existing keypair, so generate a new keypair for peer_id
        // let peer_id = irn_api::auth::peer_id(&key.verifying_key());
        let peer_id = irn_network::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let node_addr = node_addr
            .parse::<SocketAddr>()
            .map_err(|_| StorageError::WrongNodeAddress(node_addr))?;
        let address = (peer_id, irn_network::socketaddr_to_multiaddr(node_addr));
        let namespace = Auth::from_secret(namespace_secret.as_bytes(), namespace.as_bytes())
            .map_err(|_| StorageError::WrongNamespace(namespace))?;
        let client = Client::new(irn_api::client::Config {
            key,
            nodes: [address].into(),
            shadowing_nodes: Default::default(),
            shadowing_factor: 0.0,
            request_timeout: REQUEST_TIMEOUT,
            max_operation_time: MAX_OPERATION_TIME,
            connection_timeout: CONNECTION_TIMEOUT,
            udp_socket_count: UDP_SOCKET_COUNT,
            namespaces: vec![namespace.clone()],
        })?;

        Ok(Self {
            client,
            namespace: namespace.public_key(),
        })
    }

    /// Calculate unixtimestamp based on the record TTL and the current time
    pub fn calculate_ttl(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Failed to get time since epoch")
            .as_secs();
        now + RECORDS_TTL.as_secs()
    }

    /// Create a key from the namespace and the bytes
    fn key(&self, bytes: Vec<u8>) -> Key {
        Key {
            namespace: Some(self.namespace),
            bytes,
        }
    }

    /// Set a value in the storage
    pub async fn set(&self, key: String, value: Vec<u8>) -> Result<(), StorageError> {
        self.client
            .set(
                self.key(key.as_bytes().into()),
                value,
                Some(self.calculate_ttl()),
            )
            .await
            .map_err(StorageError::IrnClientError)
    }

    /// Get a value from the storage
    pub async fn get(&self, key: String) -> Result<Option<String>, StorageError> {
        let result = self.client.get(self.key(key.as_bytes().into())).await;

        match result {
            Ok(Some(data)) => match String::from_utf8(data) {
                Ok(string) => Ok(Some(string)),
                Err(e) => Err(StorageError::Utf8Error(e)),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_calculate_ttl() {
        let irn = Irn::new(
            "127.0.0.1:1".into(),
            "2SjlbfXx6md6337H63KjOEFlv4XP5g2dl7Qam6ot84o=".into(),
            "test_namespace".into(),
            "namespace_secret".into(),
        )
        .unwrap();

        let ttl = irn.calculate_ttl();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        assert!(ttl > now);
        assert_eq!(ttl, now + RECORDS_TTL.as_secs());
    }

    #[tokio::test]
    #[ignore]
    async fn test_set_get() {
        let irn = Irn::new(
            "127.0.0.1:3011".into(),
            "2SjlbfXx6md6337H63KjOEFlv4XP5g2dl7Qam6ot84o=".into(),
            "test_namespace".into(),
            "namespace_secret".into(),
        )
        .unwrap();

        let key = "test_key".to_string();
        let value = "test_value".to_string().into_bytes();
        irn.set(key.clone(), value.clone()).await.unwrap();
        let result = irn.get(key.clone()).await.unwrap().unwrap();
        assert_eq!(value, result.into_bytes());
    }
}
