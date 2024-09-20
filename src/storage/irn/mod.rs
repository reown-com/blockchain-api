use {
    super::StorageError,
    irn_api::{
        auth::{Auth, PublicKey},
        Client, Key,
    },
    irn_rpc::identity::Keypair,
    serde::Deserialize,
    std::{net::SocketAddr, time::Duration},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_OPERATION_TIME: Duration = Duration::from_secs(3);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);
const RECORDS_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 30); // 30 days
const UDP_SOCKET_COUNT: usize = 1;

/// IRN storage operation type
#[derive(Debug)]
pub enum OperationType {
    Hset,
    Hget,
    Hfields,
    Hdel,
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Hset => "hset",
            OperationType::Hget => "hget",
            OperationType::Hfields => "hfields",
            OperationType::Hdel => "hdel",
        }
    }
}

impl From<OperationType> for String {
    fn from(value: OperationType) -> Self {
        value.as_str().to_string()
    }
}

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
        let keypair = Keypair::ed25519_from_bytes(
            data_encoding::BASE64
                .decode(key_base64.as_bytes())
                .map_err(|_| StorageError::WrongKey(key_base64.clone()))?,
        )
        .map_err(|_| StorageError::WrongKey(key_base64))?;
        // Generating peer_id. This should be replaced by the actual peer_id
        // in a future
        let peer_id = irn_rpc::identity::Keypair::generate_ed25519()
            .public()
            .to_peer_id();
        let node_addr = node_addr
            .parse::<SocketAddr>()
            .map_err(|_| StorageError::WrongNodeAddress(node_addr))?;
        let address = (peer_id, irn_rpc::quic::socketaddr_to_multiaddr(node_addr));
        let namespace = Auth::from_secret(namespace_secret.as_bytes(), namespace.as_bytes())
            .map_err(|_| StorageError::WrongNamespace(namespace))?;
        let client = Client::new(irn_api::client::Config {
            keypair,
            nodes: [address].into(),
            shadowing_nodes: Default::default(),
            shadowing_factor: 0.0,
            request_timeout: REQUEST_TIMEOUT,
            max_operation_time: MAX_OPERATION_TIME,
            connection_timeout: CONNECTION_TIMEOUT,
            udp_socket_count: UDP_SOCKET_COUNT,
            namespaces: vec![namespace.clone()],
            shadowing_default_namespace: None,
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
            .set(self.key(key.as_bytes().into()), value, self.calculate_ttl())
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

    /// Delete a value from the storage
    pub async fn delete(&self, key: String) -> Result<(), StorageError> {
        self.client
            .del(self.key(key.as_bytes().into()))
            .await
            .map_err(StorageError::IrnClientError)
    }

    /// Set the hasmap value in the storage
    pub async fn hset(
        &self,
        key: String,
        field: String,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        self.client
            .hset(
                self.key(key.as_bytes().into()),
                field.as_bytes().into(),
                value,
                self.calculate_ttl(),
            )
            .await
            .map_err(StorageError::IrnClientError)
    }

    /// Get the hashmap value from the storage
    pub async fn hget(&self, key: String, field: String) -> Result<Option<String>, StorageError> {
        let result = self
            .client
            .hget(self.key(key.as_bytes().into()), field.as_bytes().into())
            .await;

        match result {
            Ok(Some(data)) => match String::from_utf8(data) {
                Ok(string) => Ok(Some(string)),
                Err(e) => Err(StorageError::Utf8Error(e)),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete the hashmap value from the storage
    pub async fn hdel(&self, key: String, field: String) -> Result<(), StorageError> {
        self.client
            .hdel(self.key(key.as_bytes().into()), field.as_bytes().into())
            .await
            .map_err(StorageError::IrnClientError)
    }

    /// Get all the hashmap fields from the storage
    pub async fn hfields(&self, key: String) -> Result<Vec<String>, StorageError> {
        let result = self.client.hfields(self.key(key.as_bytes().into())).await?;
        let fields = result
            .into_iter()
            .map(String::from_utf8)
            .collect::<Result<Vec<String>, _>>()?;
        Ok(fields)
    }

    /// Get all the hashmap values from the storage
    pub async fn hvals(&self, key: String) -> Result<Vec<String>, StorageError> {
        let result = self.client.hvals(self.key(key.as_bytes().into())).await?;
        let fields = result
            .into_iter()
            .map(String::from_utf8)
            .collect::<Result<Vec<String>, _>>()?;
        Ok(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_irn_client_calculate_ttl() {
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

    /// Ignoring this test by default to use it for local cluster testing only
    #[ignore]
    #[tokio::test]
    async fn test_irn_client_set_get_del() {
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

        // Get the value from the correct key
        let result = irn.get(key.clone()).await.unwrap().unwrap();
        assert_eq!(value, result.into_bytes());

        // Get the value from the wrong key
        let result = irn.get("wrong_key".into()).await.unwrap();
        assert_eq!(None, result);

        // Delete the value
        irn.delete(key.clone()).await.unwrap();

        // Get the value after deletion
        let result = irn.get(key.clone()).await.unwrap();
        assert_eq!(None, result);
    }

    /// Ignoring this test by default to use it for local cluster testing only
    #[ignore]
    #[tokio::test]
    async fn test_irn_client_hashmap() {
        let irn = Irn::new(
            "127.0.0.1:3011".into(),
            "2SjlbfXx6md6337H63KjOEFlv4XP5g2dl7Qam6ot84o=".into(),
            "test_namespace".into(),
            "namespace_secret".into(),
        )
        .unwrap();

        let key = "test_key".to_string();
        let field = "test_field".to_string();
        let value = "test_value".to_string().into_bytes();

        // Set and get the hashmap field value
        irn.hset(key.clone(), field.clone(), value.clone())
            .await
            .unwrap();
        let result = irn.hget(key.clone(), field.clone()).await.unwrap().unwrap();
        assert_eq!(value, result.into_bytes());

        // Get hasmap fields list
        let fields = irn.hfields(key.clone()).await.unwrap();
        assert_eq!(vec![field.clone()], fields);

        // Get hasmap values list
        let values = irn.hvals(key.clone()).await.unwrap();
        assert_eq!(
            vec![value.clone()],
            values
                .iter()
                .map(|v| v.clone().into_bytes())
                .collect::<Vec<Vec<u8>>>()
        );

        // Delete the hashmap field
        irn.hdel(key.clone(), field.clone()).await.unwrap();
        let result = irn.hget(key.clone(), field.clone()).await.unwrap();
        assert_eq!(None, result);
        let fields = irn.hfields(key.clone()).await.unwrap();
        assert_eq!(Vec::<String>::new(), fields);
    }
}
