use {
    axum::response::IntoResponse,
    hyper::{Body, Response, StatusCode},
    serde::Deserialize,
};

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
}

pub fn new_error_response(field: String, description: String) -> ErrorResponse {
    ErrorResponse {
        status: "FAILED".to_string(),
        reasons: vec![ErrorReason { field, description }],
    }
}

// pub fn field_validation_error(
//     field: impl Into<String>,
//     description: impl Into<String>,
// ) -> impl IntoResponse {
//     new_error_response(vec![ErrorReason {
//         field: field.into(),
//         description: description.into(),
//     }])
// }

// pub fn handshake_error(field: impl Into<String>, description: impl
// Into<String>) -> Response<Body> {     new_error_response(
//         vec![ErrorReason {
//             field: field.into(),
//             description: description.into(),
//         }],
//         StatusCode::FORBIDDEN,
//     )
//     .into_response()
// }

#[derive(serde::Serialize)]
pub struct SuccessResponse {
    status: String,
}
