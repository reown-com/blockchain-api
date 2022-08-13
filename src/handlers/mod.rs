pub mod health;

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

#[derive(serde::Serialize)]
pub struct SuccessResponse {
    status: String,
}
