use {
    super::{
        CheckPosTxError, CheckTransactionParams, CheckTransactionResult, SupportedNamespaces,
        TransactionId, TransactionStatus,
    },
    crate::handlers::wallet::pos::{evm::get_transaction_status, solana::get_transaction_status as solana_get_transaction_status},
    crate::state::AppState,
    axum::extract::State,
    std::{str::FromStr, sync::Arc},
};

const DEFAULT_CHECK_IN: usize = 1000;

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    params: CheckTransactionParams,
) -> Result<CheckTransactionResult, CheckPosTxError> {
    let transaction_id = TransactionId::try_from(params.id)
        .map_err(|e| CheckPosTxError::Validation(e.to_string()))?;

    let namespace = SupportedNamespaces::from_str(transaction_id.chain_id().namespace())
        .map_err(|e| CheckPosTxError::Validation(e.to_string()))?;

    match namespace {
        SupportedNamespaces::Eip155 => {
            let status = get_transaction_status(
                state,
                &project_id,
                &params.send_result,
                transaction_id.chain_id(),
            )
            .await
            .map_err(|e| CheckPosTxError::Validation(e.to_string()))?;

            match status {
                TransactionStatus::Pending => Ok(CheckTransactionResult {
                    status,
                    check_in: Some(DEFAULT_CHECK_IN),
                }),
                _ => Ok(CheckTransactionResult {
                    status,
                    check_in: None,
                }),
            }
        },
        SupportedNamespaces::Solana => {
            let status = solana_get_transaction_status(
                state,
                &project_id,
                &params.send_result,
                transaction_id.chain_id(),
            )
            .await
            .map_err(|e| CheckPosTxError::Validation(e.to_string()))?;

            match status {
                TransactionStatus::Pending => Ok(CheckTransactionResult {
                    status,
                    check_in: Some(DEFAULT_CHECK_IN),
                }),
                _ => Ok(CheckTransactionResult {
                    status,
                    check_in: None,
                }),
            }
        }
    }
}
