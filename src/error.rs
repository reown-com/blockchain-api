pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error(transparent)]
    EnvyError(#[from] envy::Error),

    #[error("Chain not found despite previous validation")]
    ChainNotFound,

    #[error("Transport error: {0}")]
    TransportError(#[from] hyper::Error),

    #[error("Request::builder() failed: {0}")]
    RequestBuilderError(#[from] hyper::http::Error),
}
