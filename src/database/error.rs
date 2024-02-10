#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Bad argument were provided for the database helper: {0}")]
    BadArgument(String),
    #[error("Address required: {0}")]
    AddressRequired(String),
    #[error("{0:?}")]
    SerdeJson(#[from] serde_json::Error),
}
