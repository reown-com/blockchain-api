use {
    super::StorageError,
    serde::Deserialize,
    std::{collections::HashSet, str::FromStr, time::Duration},
    wc::metrics::{self, enum_ordinalize::Ordinalize, Enum},
    wcn_replication::{
        auth::{client_key_from_secret, peer_id, PublicKey},
        identity::Keypair,
        storage::{auth::Auth, Entry, Key, MapEntry},
        Config as WcnConfig, Driver, PeerAddr,
    },
};

const MAX_OPERATION_TIME: Duration = Duration::from_secs(3);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);
const RECORDS_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 30); // 30 days

/// IRN storage operation type
#[derive(Clone, Copy, Debug, Ordinalize)]
pub enum OperationType {
    Hset,
    Hget,
    Hscan,
    Hdel,
    Set,
    Get,
}

impl metrics::Enum for OperationType {
    fn as_str(&self) -> &'static str {
        match self {
            OperationType::Hset => "hset",
            OperationType::Hget => "hget",
            OperationType::Hscan => "hscan",
            OperationType::Hdel => "hdel",
            OperationType::Set => "set",
            OperationType::Get => "get",
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
    pub nodes: Option<Vec<String>>,
    pub key: Option<String>,
    pub namespace: Option<String>,
    pub namespace_secret: Option<String>,
}

#[derive(Clone)]
pub struct Irn {
    driver: Driver,
    namespace: PublicKey,
}

impl Irn {
    pub async fn new(
        key: String,
        nodes: Vec<String>,
        namespace: String,
        namespace_secret: String,
    ) -> Result<Self, StorageError> {
        let client_key =
            client_key_from_secret(key.as_bytes()).map_err(StorageError::WcnAuthError)?;

        // Safe unwrap as the key is guaranteed to be valid since its created by the
        // `client_key_from_secret``
        let keypair = Keypair::ed25519_from_bytes(client_key.to_bytes())
            .expect("Failed to create keypair from ed25519 client key");

        // Verify and log client public key for debugging purposes.
        let public_key = client_key.verifying_key();
        let peer_id = peer_id(&public_key);
        let public_key = data_encoding::BASE64.encode(public_key.as_bytes());

        tracing::info!(%peer_id, %public_key, "IRN client key");

        let nodes = Self::parse_node_addresses(nodes)?;
        let namespace = Auth::from_secret(namespace_secret.as_bytes(), namespace.as_bytes())
            .map_err(|_| StorageError::WrongNamespace(namespace))?;

        let config = WcnConfig::new(nodes)
            .with_keypair(keypair)
            .with_namespaces(vec![namespace.clone()])
            .with_connection_timeout(CONNECTION_TIMEOUT)
            .with_operation_timeout(MAX_OPERATION_TIME);

        let driver = Driver::new(config)
            .await
            .map_err(StorageError::WcnDriverCreationError)?;

        Ok(Self {
            driver,
            namespace: namespace.public_key(),
        })
    }

    fn parse_node_addresses(addresses: Vec<String>) -> Result<HashSet<PeerAddr>, StorageError> {
        let mut nodes = HashSet::new();
        for address in addresses {
            let addr = PeerAddr::from_str(&address)
                .map_err(|_| StorageError::WrongNodeAddress(address))?;
            nodes.insert(addr);
        }
        Ok(nodes)
    }

    /// Create a key from the namespace and the bytes
    fn key(&self, key: Vec<u8>) -> Key {
        Key::private(&self.namespace, key)
    }

    /// Set a value in the storage
    pub async fn set(&self, key: String, value: Vec<u8>) -> Result<(), StorageError> {
        self.driver
            .set(Entry::new(
                self.key(key.as_bytes().into()),
                value,
                RECORDS_TTL,
            ))
            .await
            .map_err(StorageError::WcnClientError)
    }

    /// Get a value from the storage
    pub async fn get(&self, key: String) -> Result<Option<Vec<u8>>, StorageError> {
        let result = self.driver.get(self.key(key.as_bytes().into())).await;

        match result {
            Ok(Some(record)) => Ok(Some(record.value)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete a value from the storage
    pub async fn delete(&self, key: String) -> Result<(), StorageError> {
        self.driver
            .del(self.key(key.as_bytes().into()))
            .await
            .map_err(StorageError::WcnClientError)
    }

    /// Set the hasmap value in the storage
    pub async fn hset(
        &self,
        key: String,
        field: String,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        self.driver
            .hset(MapEntry::new(
                self.key(key.as_bytes().to_vec()),
                field.as_bytes(),
                value,
                RECORDS_TTL,
            ))
            .await
            .map_err(StorageError::WcnClientError)
    }

    /// Get the hashmap value from the storage
    pub async fn hget(&self, key: String, field: String) -> Result<Option<Vec<u8>>, StorageError> {
        let result = self
            .driver
            .hget(self.key(key.as_bytes().into()), field.as_bytes().into())
            .await;

        match result {
            Ok(Some(record)) => Ok(Some(record.value)),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete the hashmap value from the storage
    pub async fn hdel(&self, key: String, field: String) -> Result<(), StorageError> {
        self.driver
            .hdel(self.key(key.as_bytes().into()), field.as_bytes().into())
            .await
            .map_err(StorageError::WcnClientError)
    }

    /// Get all the hashmap ((field, value) cursor) from the storage
    pub async fn hscan(
        &self,
        key: String,
        count: u32,
        cursor: Option<Vec<u8>>,
    ) -> Result<(Vec<(String, Vec<u8>)>, Option<Vec<u8>>), StorageError> {
        let result = self
            .driver
            .hscan(self.key(key.as_bytes().into()), count, cursor)
            .await
            .map(|resp| {
                let cursor = resp.next_page_cursor().cloned();
                let records = resp.records.into_iter().map(|rec| (rec.field, rec.value));

                (records, cursor)
            })
            .map_err(StorageError::WcnClientError)?;

        let (records, next_cursor) = result;
        let fields_values = records
            .map(|(field_bytes, value_bytes)| {
                let field_string =
                    String::from_utf8(field_bytes).map_err(StorageError::Utf8Error)?;
                Ok((field_string, value_bytes))
            })
            .collect::<Result<Vec<(String, Vec<u8>)>, StorageError>>()?;

        Ok((fields_values, next_cursor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ignoring this test by default to use it for local cluster testing only
    #[ignore]
    #[tokio::test]
    async fn test_irn_client_set_get_del() {
        let irn = Irn::new(
            "2SjlbfXx6md6337H63KjOEFlv4XP5g2dl7Qam6ot84o=".into(),
            vec!["/ip4/127.0.0.1/udp/3011/quic-v1".into()],
            "test_namespace".into(),
            "namespace_secret".into(),
        )
        .await
        .unwrap();

        let key = "test_key".to_string();
        let value = "test_value".to_string().into_bytes();
        irn.set(key.clone(), value.clone()).await.unwrap();

        // Get the value from the correct key
        let result = irn.get(key.clone()).await.unwrap().unwrap();
        assert_eq!(value, result);

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
        let addr =
            "12D3KooWDJrGKPuU1vJLBZv2UXfcZvdBprUgAkjvkUET7q2PzwPp-/ip4/127.0.0.1/udp/3011/quic-v1";

        let irn = Irn::new(
            "2SjlbfXx6md6337H63KjOEFlv4XP5g2dl7Qam6ot84o=".into(),
            vec![addr.into()],
            "test_namespace".into(),
            "namespace_secret".into(),
        )
        .await
        .unwrap();

        let key = "test_key".to_string();
        let field = "test_field".to_string();
        let value = "test_value".to_string().into_bytes();

        // Set and get the hashmap field value
        irn.hset(key.clone(), field.clone(), value.clone())
            .await
            .unwrap();
        let result = irn.hget(key.clone(), field.clone()).await.unwrap().unwrap();
        assert_eq!(value, result);

        // Get hashmap scan list
        let (fields_values, _) = irn.hscan(key.clone(), 5, None).await.unwrap();
        assert_eq!(
            (field.clone(), value),
            fields_values.first().unwrap().clone()
        );

        // Delete the hashmap field
        irn.hdel(key.clone(), field.clone()).await.unwrap();
        let result = irn.hget(key.clone(), field.clone()).await.unwrap();
        assert_eq!(None, result);
        let (fields_values, _) = irn.hscan(key.clone(), 5, None).await.unwrap();
        assert_eq!(Vec::<(String, Vec<u8>)>::new(), fields_values);
    }
}
