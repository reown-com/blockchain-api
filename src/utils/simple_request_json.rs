use async_trait::async_trait;
use axum::extract::rejection::JsonRejection;
use axum::http::{HeaderValue, Request};
use axum::Json;
use axum::{body::HttpBody, extract::FromRequest, BoxError};
use serde::de::DeserializeOwned;

/// Same as axum::Json but doesn't care what the Content-Type header is
#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct SimpleRequestJson<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for SimpleRequestJson<T>
where
    T: DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(mut req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        // Fake the header to make the extractor happy
        req.headers_mut()
            .entry("content-type")
            .or_insert(HeaderValue::from_static("application/json"));

        let inner = Json::from_request(req, state).await?;

        Ok(Self(inner.0))
    }
}
