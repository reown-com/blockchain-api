pub mod build_transaction;
pub mod check_transaction;
pub mod evm;

use {
    crate::state::AppState,
    axum::extract::State,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::sync::Arc,
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum BuildPosTxError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl BuildPosTxError {
    pub fn is_internal(&self) -> bool {
        matches!(self, BuildPosTxError::Internal(_))
    }
}

#[derive(Debug, Error)]
pub enum CheckPosTxError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl CheckPosTxError {
    pub fn is_internal(&self) -> bool {
        matches!(self, CheckPosTxError::Internal(_))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTransactionParams {
    pub asset: String,
    pub amount: String,
    pub recipient: String,
    pub sender: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTransactionResult {
    pub transaction_rpc: TransactionRpc,
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRpc {
    pub method: String,
    pub params: Value,
}

#[async_trait::async_trait]
pub trait TransactionBuilder {
    fn namespace(&self) -> &'static str;
    async fn build(
        &self,
        state: State<Arc<AppState>>,
        project_id: String,
        params: BuildTransactionParams,
    ) -> Result<BuildTransactionResult, BuildPosTxError>;
}


    