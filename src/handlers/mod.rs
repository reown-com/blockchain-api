use hyper::{Body, Response, StatusCode};
use serde::Deserialize;

pub mod health;
pub mod metrics;
pub mod proxy;

#[derive(Deserialize)]
pub struct RPCQueryParams {
    #[serde(rename = "chainId")]
    pub chain_id: String,
    #[serde(rename = "projectId")]
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

#[derive(serde::Serialize)]
pub struct SuccessResponse {
    status: String,
}

impl warp::Reply for ErrorResponse {
    fn into_response(self) -> Response<Body> {
        let error = serde_json::to_string(&self).unwrap();
        Response::builder()
            .status(self.code)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(error))
            .unwrap()
    }
}
