use {
    async_trait::async_trait,
    axum::{extract::FromRequestParts, http::request::Parts},
    hyper::StatusCode,
};

pub struct Method(pub hyper::http::Method);

#[async_trait]
impl<S> FromRequestParts<S> for Method
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Method(parts.method.clone()))
    }
}
