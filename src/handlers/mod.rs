use hyper::{Body, Response, StatusCode};
use serde::Deserialize;
use warp::Reply;

pub mod health;
pub mod metrics;
pub mod proxy;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcQueryParams {
    pub chain_id: String,
    pub project_id: String,
}

#[derive(serde::Serialize)]
pub struct ErrorReason {
    pub field: String,
    pub description: String,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub reasons: Vec<ErrorReason>,
    #[serde(skip_serializing)]
    pub code: StatusCode,
}

pub fn new_error_response(reasons: Vec<ErrorReason>, code: StatusCode) -> ErrorResponse {
    ErrorResponse {
        status: "FAILED".to_string(),
        reasons,
        code,
    }
}

pub fn field_validation_error(
    field: impl Into<String>,
    description: impl Into<String>,
) -> Response<Body> {
    new_error_response(
        vec![ErrorReason {
            field: field.into(),
            description: description.into(),
        }],
        StatusCode::BAD_REQUEST,
    )
    .into_response()
}

#[derive(serde::Serialize)]
pub struct SuccessResponse {
    status: String,
}

impl Reply for ErrorResponse {
    fn into_response(self) -> Response<Body> {
        let error = serde_json::to_string(&self).unwrap_or_default();
        Response::builder()
            .status(self.code)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(error))
            .unwrap()
    }
}
