use crate::storage::error::StorageError;
use cerberus::registry::RegistryError;
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ProjectStorageError {
    #[error("registry error: {0}")]
    Registry(#[from] RegistryError),

    #[error("cache error: {0}")]
    Cache(#[from] StorageError),
}

#[derive(Debug, Clone, Serialize, Deserialize, ThisError)]
pub enum ProjectDataError {
    #[error("Project not found in registry")]
    NotFound,

    #[error("Registry configuration error")]
    RegistryConfigError,
}
