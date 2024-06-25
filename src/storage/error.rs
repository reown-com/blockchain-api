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
    #[error("error on deserialize data")]
    Deserialize,
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
    /// IRN network errors
    #[error("IRN network error: {0}")]
    IrnNetworkError(#[from] irn_network::Error),
    /// An unexpected error occurred
    #[error("{0:?}")]
    Other(String),
}
