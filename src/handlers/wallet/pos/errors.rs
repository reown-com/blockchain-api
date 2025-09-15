use {
    thiserror::Error,
    base64::DecodeError,
    crate::utils::crypto::CryptoUitlsError,
};

#[derive(Debug, Error)]
pub enum InternalError {
    #[error("Invalid provider URL: {0}")]
    InvalidProviderUrl(String),

    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("{0}")]
    Internal(String),
}

impl InternalError {
    pub fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            InternalError::InvalidProviderUrl(_) => -12001,
            InternalError::RpcError(_) => -12002,
            InternalError::Internal(_) => -12999,
        }
    }
}

#[derive(Debug, Error)]
pub enum BuildPosTxsError {
    #[error("Validation error: {0}")]
    Validation(#[source] ValidationError),

    #[error("Execution error: {0}")]
    Execution(#[source] ExecutionError),

    #[error("Internal error: {0}")]
    Internal(#[source] InternalError),
}

impl BuildPosTxsError {
    pub fn is_internal(&self) -> bool {
        matches!(self, BuildPosTxsError::Internal(_))
    }

    pub fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            BuildPosTxsError::Validation(v) => v.to_json_rpc_error_code(),
            BuildPosTxsError::Execution(e) => e.to_json_rpc_error_code(),
            BuildPosTxsError::Internal(i) => i.to_json_rpc_error_code(),
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    #[error("Invalid Asset: {0}")]
    InvalidAsset(String),
    #[error("Invalid Recipient: {0}")]
    InvalidRecipient(String),
    #[error("Invalid Sender: {0}")]
    InvalidSender(String),
    #[error("Invalid Amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid Address: {0}")]
    InvalidAddress(String),
    #[error("Invalid Wallet Response: {0}")]
    InvalidWalletResponse(String),
    #[error("Invalid Transaction ID: {0}")]
    InvalidTransactionId(String),
}

impl ValidationError {
    pub fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            ValidationError::InvalidAsset(_) => -10001,
            ValidationError::InvalidRecipient(_) => -10002,
            ValidationError::InvalidSender(_) => -10003,
            ValidationError::InvalidAmount(_) => -10004,
            ValidationError::InvalidAddress(_) => -10005,
            ValidationError::InvalidWalletResponse(_) => -10006,
            ValidationError::InvalidTransactionId(_) => -10007,
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum ExecutionError {
    #[error("Unable to estimate gas: {0}")]
    GasEstimation(String),

}

impl ExecutionError {
    pub fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            ExecutionError::GasEstimation(_) => -11001,
        }
    }
}

#[derive(Debug, Error)]
pub enum CheckPosTxError {
    #[error("Validation error: {0}")]
    Validation(#[source] ValidationError),

    #[error("Internal error: {0}")]
    Internal(#[source] InternalError),
}

impl CheckPosTxError {
    pub fn is_internal(&self) -> bool {
        matches!(self, CheckPosTxError::Internal(_))
    }

    pub fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            CheckPosTxError::Validation(v) => v.to_json_rpc_error_code(),
            CheckPosTxError::Internal(i) => i.to_json_rpc_error_code(),
        }
    }
}

#[derive(Debug, Error)]
pub enum SupportedNetworksError {
    #[error("Internal error: {0}")]
    Internal(String),
}

impl SupportedNetworksError {
    pub fn is_internal(&self) -> bool {
        matches!(self, SupportedNetworksError::Internal(_))
    }

    pub fn to_json_rpc_error_code(&self) -> i32 {
        -12999
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

impl TransactionIdError {
    pub fn to_json_rpc_error_code(&self) -> i32 {
        match self {
            TransactionIdError::InvalidBase64(_) => -13001,
            TransactionIdError::InvalidFormat(_) => -13002,
            TransactionIdError::InvalidChainId(_) => -13003,
        }
    }
}