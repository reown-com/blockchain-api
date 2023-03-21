use {
    crate::storage::error::StorageError,
    async_trait::async_trait,
    serde::{de::DeserializeOwned, Serialize},
    std::{fmt::Debug, time::Duration},
};

pub mod error;
pub mod redis;

/// The Result type returned by Storage functions
pub type StorageResult<T> = Result<T, StorageError>;

#[async_trait]
pub trait KeyValueStorage<T>: 'static + Send + Sync + Debug
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// Retrieve the data associated with the given key.
    async fn get(&self, key: &str) -> StorageResult<Option<T>>;

    /// Set the value for the given key.
    async fn set(&self, key: &str, value: &T, ttl: Option<Duration>) -> StorageResult<()>;

    /// Set the value for the given key. Assumes the data is already serialized.
    async fn set_serialized(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<Duration>,
    ) -> StorageResult<()>;

    /// Delete the value associated with the given key.
    async fn del(&self, key: &str) -> StorageResult<()>;
}

/// Holder the type of data will be serialized to be stored.
pub type Data = Vec<u8>;

pub fn serialize<T>(data: &T) -> StorageResult<Data>
where
    T: Serialize,
{
    rmp_serde::to_vec(data).map_err(|_| StorageError::Serialize)
}

pub fn deserialize<T>(data: &[u8]) -> StorageResult<T>
where
    T: DeserializeOwned,
{
    rmp_serde::from_slice(data).map_err(|_| StorageError::Deserialize)
}
