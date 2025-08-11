use {
    serde::{Deserialize, Serialize},
    thiserror::Error as ThisError,
};

#[derive(Debug, Clone, Serialize, Deserialize, ThisError)]
pub enum ProjectDataError {
    #[error("Project not found in registry")]
    NotFound,

    #[error("Registry configuration error")]
    RegistryConfigError,
}
