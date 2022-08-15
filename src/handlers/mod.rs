use hyper::{Response, StatusCode, Body};

pub mod health;
pub mod proxy;

#[derive(serde::Serialize)]
pub struct ErrorReason {
    pub field: String,
    pub description: String,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub reasons: Vec<ErrorReason>,
    pub code: StatusCode,
}

pub fn new_error_response(reasons: Vec<ErrorReason>, code: StatusCode) -> ErrorResponse {
    ErrorResponse {
        status: "FAILED".to_string(),
        reasons,
        code
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
