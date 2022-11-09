use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

use crate::storage::error::StorageError;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

pub mod error;
pub mod redis;

/// The Result type returned by Storage functions
pub type StorageResult<T> = Result<T, StorageError>;

#[async_trait]
pub trait SetStorage<T>: 'static + Send + Sync + Debug
where
    T: Serialize + DeserializeOwned + Hash + PartialEq + Eq + Send + Sync + Clone + Debug,
{
    /// Retrieve data related with the given key.
    async fn sget(&self, key: &str) -> StorageResult<HashSet<T>>;

    /// Add the value into the set for the given key.
    async fn sadd(&self, key: &str, value: &[&T], ttl: Option<Duration>) -> StorageResult<()>;

    /// Remove the value from the set of the given key.
    async fn srem(&self, key: &str, value: &T) -> StorageResult<()>;
}

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
