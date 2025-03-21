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
        req.headers_mut()
            .insert("content-type", HeaderValue::from_static("application/json"));

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

    #[derive(Debug, Deserialize, PartialEq)]
    struct NestedStruct {
        inner: TestStruct,
        number: i32,
        array: Vec<String>,
        optional: Option<String>,
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

    #[tokio::test]
    async fn test_case_insensitive_content_type() {
        let json = json!({ "field": "test" });
        let body = Body::from(serde_json::to_string(&json).unwrap());
        let mut request = Request::new(body);
        request
            .headers_mut()
            .insert("CONTENT-TYPE", HeaderValue::from_static("text/plain"));

        let result = SimpleRequestJson::<TestStruct>::from_request(request, &()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.field, "test");
    }

    #[tokio::test]
    async fn test_multiple_content_types() {
        let json = json!({ "field": "test" });
        let body = Body::from(serde_json::to_string(&json).unwrap());
        let mut request = Request::new(body);
        request
            .headers_mut()
            .append("content-type", HeaderValue::from_static("text/plain"));
        request
            .headers_mut()
            .append("content-type", HeaderValue::from_static("application/xml"));

        let result = SimpleRequestJson::<TestStruct>::from_request(request, &()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.field, "test");
    }

    #[tokio::test]
    async fn test_empty_json() {
        let json = json!({});
        let body = Body::from(serde_json::to_string(&json).unwrap());
        let request = Request::new(body);

        let result = SimpleRequestJson::<TestStruct>::from_request(request, &()).await;
        assert!(result.is_err()); // Should fail because TestStruct requires a field
    }

    #[tokio::test]
    async fn test_nested_json() {
        let json = json!({
            "inner": { "field": "test" },
            "number": 42,
            "array": ["a", "b", "c"],
            "optional": null
        });
        let body = Body::from(serde_json::to_string(&json).unwrap());
        let request = Request::new(body);

        let result = SimpleRequestJson::<NestedStruct>::from_request(request, &()).await;
        assert!(result.is_ok());
        let value = result.unwrap().0;
        assert_eq!(value.inner.field, "test");
        assert_eq!(value.number, 42);
        assert_eq!(value.array, vec!["a", "b", "c"]);
        assert_eq!(value.optional, None);
    }

    #[tokio::test]
    async fn test_large_json() {
        // Create a large JSON object with 1000 fields
        let mut json = serde_json::Map::new();
        for i in 0..1000 {
            json.insert(
                format!("field{}", i),
                serde_json::Value::String(format!("value{}", i)),
            );
        }
        let json = serde_json::Value::Object(json);
        let body = Body::from(serde_json::to_string(&json).unwrap());
        let request = Request::new(body);

        let result = SimpleRequestJson::<serde_json::Value>::from_request(request, &()).await;
        assert!(result.is_ok());
        let value = result.unwrap().0;
        assert!(value.as_object().unwrap().len() == 1000);
    }

    #[tokio::test]
    async fn test_non_utf8_body() {
        // Create a body with invalid UTF-8
        let body = Body::from(vec![0xFF, 0xFE, 0x00, 0x00]);
        let request = Request::new(body);

        let result = SimpleRequestJson::<TestStruct>::from_request(request, &()).await;
        assert!(result.is_err());
    }
}
