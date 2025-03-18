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
        // Always set the header to application/json, regardless of what was there before
        req.headers_mut().insert(
            "content-type",
            HeaderValue::from_static("application/json"),
        );

        let inner = Json::from_request(req, state).await?;

        Ok(Self(inner.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestStruct {
        field: String,
    }

    #[tokio::test]
    async fn test_with_content_type() {
        let json = json!({ "field": "test" });
        let body = Body::from(serde_json::to_string(&json).unwrap());
        let mut request = Request::new(body);
        request
            .headers_mut()
            .insert("content-type", HeaderValue::from_static("application/json"));

        let result = SimpleRequestJson::<TestStruct>::from_request(request, &()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.field, "test");
    }

    #[tokio::test]
    async fn test_without_content_type() {
        let json = json!({ "field": "test" });
        let body = Body::from(serde_json::to_string(&json).unwrap());
        let request = Request::new(body);

        let result = SimpleRequestJson::<TestStruct>::from_request(request, &()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.field, "test");
    }

    #[tokio::test]
    async fn test_with_wrong_content_type() {
        let json = json!({ "field": "test" });
        let body = Body::from(serde_json::to_string(&json).unwrap());
        let mut request = Request::new(body);
        request
            .headers_mut()
            .insert("content-type", HeaderValue::from_static("text/plain"));

        let result = SimpleRequestJson::<TestStruct>::from_request(request, &()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.field, "test");
    }

    #[tokio::test]
    async fn test_invalid_json() {
        let body = Body::from("invalid json");
        let request = Request::new(body);

        let result = SimpleRequestJson::<TestStruct>::from_request(request, &()).await;
        assert!(result.is_err());
    }
}
