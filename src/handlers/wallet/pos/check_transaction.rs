use {
    super::{
        CheckPosTxError, CheckTransactionParams, CheckTransactionResult, SupportedNamespaces,
        TransactionId,
    },
    crate::handlers::wallet::pos::evm::get_transaction_status,
    crate::state::AppState,
    axum::extract::State,
    std::{str::FromStr, sync::Arc},
};

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
            let status =
                get_transaction_status(state, &project_id, &params.txid, transaction_id.chain_id())
                    .await
                    .map_err(|e| CheckPosTxError::Validation(e.to_string()))?;

            Ok(CheckTransactionResult { status })
        }
    }
}
