use {
    super::StorageError,
    serde::Deserialize,
    std::{collections::HashSet, time::Duration},
    tap::Pipe as _,
    wc::metrics::{self, enum_ordinalize::Ordinalize, Enum},
    wcn::{ClusterKey, EncryptionKey, Keypair, Namespace, NodeOperatorId, PeerAddr},
};

const MAX_OPERATION_TIME: Duration = Duration::from_secs(3);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_IDLE_CONNECTION_TIME: Duration = Duration::from_millis(500);
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

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing client key")]
    MissingClientKey,

    #[error("Failed to decode client key")]
    InvalidClientKey,

    #[error("Invalid peer address: {0}")]
    InvalidPeerAddr(String),

    #[error("Invalid cluster key")]
    InvalidClusterKey,

    #[error("Invalid operator ID: {0}")]
    InvalidOperatorId(String),

    #[error("Missing namespace")]
    MissingNamespace,

    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),

    #[error("Missing encryption secret")]
    MissingEncryptionSecret,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub client_key: Option<String>,
    pub cluster_key: Option<String>,
    pub nodes: Vec<String>,
    pub trusted_operators: Vec<String>,
    pub namespace: Option<String>,
    pub encryption_secret: Option<String>,
}

impl Config {
    /// Decodes the WCN client [`Keypair`] from a base64-encoded private ed25519
    /// key.
    pub fn client_keypair(&self) -> Result<Keypair, ConfigError> {
        let key = self
            .client_key
            .as_deref()
            .ok_or(ConfigError::MissingClientKey)?;

        data_encoding::BASE64
            .decode(key.as_bytes())
            .map_err(|_| ConfigError::MissingClientKey)?
            .pipe(Keypair::ed25519_from_bytes)
            .map_err(|_| ConfigError::MissingClientKey)
    }

    /// Decodes the 32-byte cluster encryption key from a hex string.
    pub fn cluster_key(&self) -> Result<ClusterKey, ConfigError> {
        self.cluster_key
            .as_deref()
            .ok_or(ConfigError::InvalidClusterKey)?
            .parse()
            .map_err(|_| ConfigError::InvalidClusterKey)
    }

    /// Parses a list of WCN node addresses into a list of [`PeerAddr`].
    ///
    /// Expects addresses to be in the following format:
    /// `{PeerId}-{SocketAddrV4}`.
    pub fn nodes(&self) -> Result<Vec<PeerAddr>, ConfigError> {
        let parse_addr = |addr: &String| {
            let (id, addr) = addr.split_once('-')?;
            let id = id.parse().ok()?;
            let addr = addr.parse().ok()?;
            Some(PeerAddr { id, addr })
        };

        self.nodes
            .iter()
            .map(|addr| parse_addr(addr).ok_or_else(|| ConfigError::InvalidPeerAddr(addr.clone())))
            .collect()
    }

    /// Decodes a list of trusted operators.
    ///
    /// Expects operator IDs to be hex-encoded 20-byte Ethereum addresses.
    pub fn trusted_operators(&self) -> Result<HashSet<NodeOperatorId>, ConfigError> {
        self.trusted_operators
            .iter()
            .map(|addr| {
                addr.parse()
                    .map_err(|_| ConfigError::InvalidOperatorId(addr.to_owned()))
            })
            .collect()
    }

    /// Decodes the [`Namespace`].
    pub fn namespace(&self) -> Result<Namespace, ConfigError> {
        let namespace = self
            .namespace
            .as_deref()
            .ok_or(ConfigError::MissingNamespace)?;

        namespace
            .parse()
            .map_err(|_| ConfigError::InvalidNamespace(namespace.to_owned()))
    }

    pub fn encryption_secret(&self) -> Result<&str, ConfigError> {
        self.encryption_secret
            .as_deref()
            .ok_or(ConfigError::MissingEncryptionSecret)
    }

    /// Parses this env config into [`ClientConfig`].
    pub fn parse(&self) -> Result<ClientConfig, ConfigError> {
        Ok(ClientConfig {
            keypair: self.client_keypair()?,
            cluster_key: self.cluster_key()?,
            nodes: self.nodes()?,
            trusted_operators: self.trusted_operators()?,
            namespace: self.namespace()?,
            encryption_secret: self.encryption_secret()?.to_owned(),
        })
    }
}

pub struct ClientConfig {
    pub keypair: Keypair,
    pub cluster_key: ClusterKey,
    pub nodes: Vec<PeerAddr>,
    pub trusted_operators: HashSet<NodeOperatorId>,
    pub namespace: Namespace,
    pub encryption_secret: String,
}

pub struct Irn {
    inner: wcn::Client,
    namespace: Namespace,
}

impl Irn {
    pub async fn new(config: ClientConfig) -> Result<Self, StorageError> {
        let encryption_key =
            EncryptionKey::new(config.encryption_secret.as_bytes()).map_err(StorageError::other)?;

        let inner = wcn::Client::builder(wcn::Config {
            keypair: config.keypair,
            cluster_key: config.cluster_key,
            connection_timeout: CONNECTION_TIMEOUT,
            operation_timeout: MAX_OPERATION_TIME,
            reconnect_interval: Duration::from_millis(100),
            max_concurrent_rpcs: 5000,
            max_retries: 2,
            max_idle_connection_timeout: MAX_IDLE_CONNECTION_TIME,
            nodes: config.nodes,
            trusted_operators: config.trusted_operators,
        })
        .with_encryption(encryption_key)
        .build()
        .await?;

        Ok(Self {
            inner,
            namespace: config.namespace,
        })
    }

    /// Set a value in the storage
    pub async fn set(&self, key: String, value: Vec<u8>) -> Result<(), StorageError> {
        self.inner
            .set(self.namespace, key, value, RECORDS_TTL)
            .await
            .map_err(StorageError::from)
    }

    /// Get a value from the storage
    pub async fn get(&self, key: String) -> Result<Option<Vec<u8>>, StorageError> {
        self.inner
            .get(self.namespace, key)
            .await
            .map(|rec| rec.map(|rec| rec.value))
            .map_err(StorageError::from)
    }

    /// Delete a value from the storage
    pub async fn delete(&self, key: String) -> Result<(), StorageError> {
        self.inner
            .del(self.namespace, key)
            .await
            .map_err(StorageError::from)
    }

    /// Set the hasmap value in the storage
    pub async fn hset(
        &self,
        key: String,
        field: String,
        value: Vec<u8>,
    ) -> Result<(), StorageError> {
        self.inner
            .hset(self.namespace, key, field, value, RECORDS_TTL)
            .await
            .map_err(StorageError::from)
    }

    /// Get the hashmap value from the storage
    pub async fn hget(&self, key: String, field: String) -> Result<Option<Vec<u8>>, StorageError> {
        self.inner
            .hget(self.namespace, key, field)
            .await
            .map(|rec| rec.map(|rec| rec.value))
            .map_err(StorageError::from)
    }

    /// Delete the hashmap value from the storage
    pub async fn hdel(&self, key: String, field: String) -> Result<(), StorageError> {
        self.inner
            .hdel(self.namespace, key, field)
            .await
            .map_err(StorageError::from)
    }

    /// Get all the hashmap ((field, value) cursor) from the storage
    pub async fn hscan(
        &self,
        key: String,
        count: u32,
        cursor: Option<Vec<u8>>,
    ) -> Result<(Vec<(String, Vec<u8>)>, Option<Vec<u8>>), StorageError> {
        let (records, next_cursor) = self
            .inner
            .hscan(self.namespace, key, count, cursor)
            .await
            .map(|resp| {
                let cursor = resp.next_page_cursor().map(Into::into);
                let records = resp
                    .entries
                    .into_iter()
                    .map(|entry| (entry.field, entry.record.value));

                (records, cursor)
            })
            .map_err(StorageError::from)?;

        let fields_values = records
            .map(|(field_bytes, value_bytes)| {
                let field_string =
                    String::from_utf8(field_bytes).map_err(StorageError::Utf8Error)?;

                Ok((field_string, value_bytes))
            })
            .collect::<Result<Vec<_>, StorageError>>()?;

        Ok((fields_values, next_cursor))
    }
}
