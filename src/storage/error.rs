//! Error typedefs used by this crate

use thiserror::Error as ThisError;

/// The error produced from most Storage functions
#[derive(Debug, ThisError)]
pub enum StorageError {
    /// Couldn't set the expiration for the given key
    #[error("couldn't set the expiry to the key")]
    SetExpiry,
    /// Unable to serialize data to store
    #[error("error on serialize data")]
    Serialize,
    /// Unable to deserialize data from store
    #[error("error on deserialize data: {0}")]
    Deserialize(String),
    /// Error on establishing a connection with the storage
    #[error("error on open connection")]
    Connection(String),
    /// Wrong node address
    #[error("wrong node address format: {0}")]
    WrongNodeAddress(String),
    /// Wrong key provided
    #[error("wrong key format: {0}")]
    WrongKey(String),
    /// Wrong namespace provided
    #[error("wrong namespace: {0}")]
    WrongNamespace(String),
    /// Wrong UTF8 encoding
    #[error("wrong UTF8 encoding")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    /// WCN replication client error
    #[error("WCN client error: {0}")]
    WcnClientError(#[from] wcn_replication::Error),
    #[error("WCN auth error: {0}")]
    WcnAuthError(#[from] wcn_replication::auth::Error),
    #[error("WCN driver creation error: {0}")]
    WcnDriverCreationError(#[from] wcn_replication::CreationError),
    /// An unexpected error occurred
    #[error("{0:?}")]
    Other(String),
}
