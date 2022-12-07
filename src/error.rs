use crate::project::ProjectDataError;
use crate::storage::error::StorageError;
use cerberus::registry::RegistryError;

pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error(transparent)]
    EnvyError(#[from] envy::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Project data error: {0}")]
    ProjectDataError(#[from] ProjectDataError),

    #[error("Registry error")]
    RegistryError(#[from] RegistryError),

    #[error("Storage error")]
    StorageError(#[from] StorageError),

    #[error("Chain not found despite previous validation")]
    ChainNotFound,

    #[error("Transport error: {0}")]
    TransportError(#[from] hyper::Error),

    #[error("Request::builder() failed: {0}")]
    RequestBuilderError(#[from] hyper::http::Error),

    #[error("Provider is throttling the requests")]
    Throttled,

    #[error("{0:?}")]
    Other(#[from] anyhow::Error),
}
