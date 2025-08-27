use {
    super::CheckPosTxError,
    crate::state::AppState,
    axum::extract::State,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTransactionParams {
    pub id: String,
    pub txid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTransactionResult {
    pub status: String,
}

pub async fn handler(
    _state: State<Arc<AppState>>,
    _project_id: String,
    _params: CheckTransactionParams,
) -> Result<CheckTransactionResult, CheckPosTxError> {
    Ok(CheckTransactionResult {
        status: "CONFIRMED".into(),
    })
}
