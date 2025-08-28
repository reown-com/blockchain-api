pub mod build_transaction;
pub mod check_transaction;
pub mod evm;

use {
    crate::{
        state::AppState,
        utils::crypto::{Caip2ChainId, CryptoUitlsError},
    },
    axum::extract::State,
    base64::{engine::general_purpose, DecodeError, Engine as _},
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::{convert::TryFrom, fmt::Display, sync::Arc},
    strum_macros::EnumString,
    thiserror::Error,
    uuid::Uuid,
};

const TRANSACTION_ID_DELIMITER: &str = "|";
const TRANSACTION_ID_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, EnumString, Deserialize, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum SupportedNamespaces {
    Eip155,
}

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

#[derive(Debug, Error)]
pub enum TransactionIdError {
    #[error("Invalid transaction encoding: {0}")]
    InvalidBase64(#[from] DecodeError),

    #[error("Invalid transaction format: '{0}'")]
    InvalidFormat(String),

    #[error("Invalid chain ID: {0}")]
    InvalidChainId(#[from] CryptoUitlsError),
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTransactionParams {
    pub id: String,
    pub txid: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTransactionResult {
    pub status: TransactionStatus,
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

pub struct TransactionId {
    id: String,
    chain_id: Caip2ChainId,
    version: String,
}

impl TransactionId {
    pub fn new(chain_id: &Caip2ChainId) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            chain_id: chain_id.clone(),
            version: TRANSACTION_ID_VERSION.to_string(),
        }
    }
    pub fn chain_id(&self) -> &Caip2ChainId {
        &self.chain_id
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn from(id: &str, chain_id: &Caip2ChainId) -> Self {
        Self {
            id: id.to_string(),
            chain_id: chain_id.clone(),
            version: TRANSACTION_ID_VERSION.to_string(),
        }
    }
}

impl Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted = [
            self.version.as_str(),
            self.chain_id.to_string().as_str(),
            &self.id,
        ]
        .join(TRANSACTION_ID_DELIMITER);
        write!(f, "{}", general_purpose::STANDARD_NO_PAD.encode(formatted))
    }
}

impl TryFrom<String> for TransactionId {
    type Error = TransactionIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for TransactionId {
    type Error = TransactionIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let decoded = general_purpose::STANDARD_NO_PAD.decode(value)?;
        let decoded_str = String::from_utf8(decoded)
            .map_err(|_| TransactionIdError::InvalidFormat(value.to_string()))?;

        let mut parts = decoded_str.splitn(3, TRANSACTION_ID_DELIMITER);
        let version = parts
            .next()
            .ok_or_else(|| TransactionIdError::InvalidFormat(decoded_str.clone()))?;

        if version != TRANSACTION_ID_VERSION {
            return Err(TransactionIdError::InvalidFormat(decoded_str.clone()));
        }

        let chain_id_str = parts
            .next()
            .ok_or_else(|| TransactionIdError::InvalidFormat(decoded_str.clone()))?;

        let chain_id =
            Caip2ChainId::parse(chain_id_str).map_err(TransactionIdError::InvalidChainId)?;

        let id = parts
            .next()
            .ok_or_else(|| TransactionIdError::InvalidFormat(decoded_str.clone()))?;

        Ok(TransactionId::from(id, &chain_id))
    }
}
