use {
    serde::{Deserialize, Serialize},
    thiserror::Error as ThisError,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, ThisError, Eq, PartialEq)]
pub enum ProjectDataError {
    #[error("Project not found in registry")]
    NotFound,

    #[error("Registry configuration error")]
    RegistryConfigError,

    #[error("Registry is temporarily unavailable")]
    RegistryTemporarilyUnavailable,
}
