use {
    super::{
        CheckPosTxError, CheckTransactionParams, CheckTransactionResult, SupportedNamespaces,
        TransactionId, ValidationError,
    },
    crate::handlers::wallet::pos::{
        evm::check_transaction as evm_check_transaction,
        solana::check_transaction as solana_check_transaction,
        tron::check_transaction as tron_check_transaction,
    },
    crate::state::AppState,
    axum::extract::State,
    std::{str::FromStr, sync::Arc},
};

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    params: CheckTransactionParams,
) -> Result<CheckTransactionResult, CheckPosTxError> {
    let transaction_id = TransactionId::try_from(params.id).map_err(|e| {
        CheckPosTxError::Validation(ValidationError::InvalidTransactionId(e.to_string()))
    })?;

    let namespace =
        SupportedNamespaces::from_str(transaction_id.chain_id().namespace()).map_err(|e| {
            CheckPosTxError::Validation(ValidationError::InvalidTransactionId(e.to_string()))
        })?;

    match namespace {
        SupportedNamespaces::Eip155 => {
            evm_check_transaction(
                state,
                &project_id,
                &params.send_result,
                transaction_id.chain_id(),
            )
            .await
        }
        SupportedNamespaces::Solana => {
            solana_check_transaction(
                state,
                &project_id,
                &params.send_result,
                transaction_id.chain_id(),
            )
            .await
        }
        SupportedNamespaces::Tron => {
            tron_check_transaction(
                state,
                &project_id,
                &params.send_result,
                transaction_id.chain_id(),
            )
            .await
        }
    }
}
