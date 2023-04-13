use {
    crate::error::RpcError,
    async_trait::async_trait,
    axum::{extract::FromRequestParts, http::request::Parts},
};

#[derive(Debug)]
pub enum Protocol {
    Http,
    WebSocket,
    Other(String),
}

#[async_trait]
impl<S> FromRequestParts<S> for Protocol
where
    S: Send + Sync,
{
    // type Rejection = (StatusCode, &'static str);
    type Rejection = RpcError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if let Some(scheme) = parts.uri.scheme_str() {
            return Ok(match scheme.to_lowercase().as_str() {
                "http" | "https" => Protocol::Http,
                "wss" | "ws" => Protocol::WebSocket,
                other => Self::Other(other.to_string()),
            });
        };
        return Err(RpcError::InvalidScheme);
    }
}
