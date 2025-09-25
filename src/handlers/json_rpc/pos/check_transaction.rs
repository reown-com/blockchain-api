use {
    super::{
        CheckPosTxError, CheckTransactionParams, CheckTransactionResult, SupportedNamespaces,
        TransactionId, ValidationError,
    },
    crate::{
        analytics::pos_info::PosCheckTxInfo,
        handlers::json_rpc::pos::{
        evm::check_transaction as evm_check_transaction,
        solana::check_transaction as solana_check_transaction,
        tron::check_transaction as tron_check_transaction,
    },
        state::AppState,
    },
    axum::extract::State,
    std::{str::FromStr, sync::Arc},
};

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    params: CheckTransactionParams,
) -> Result<CheckTransactionResult, CheckPosTxError> {
    let CheckTransactionParams { id, send_result } = params;
    let transaction_id = TransactionId::try_from(id).map_err(|e| {
        CheckPosTxError::Validation(ValidationError::InvalidTransactionId(e.to_string()))
    })?;

    let namespace =
        SupportedNamespaces::from_str(transaction_id.chain_id().namespace()).map_err(|e| {
            CheckPosTxError::Validation(ValidationError::InvalidTransactionId(e.to_string()))
        })?;

    let result = match namespace {
        SupportedNamespaces::Eip155 => {
            evm_check_transaction(
                state.clone(),
                &project_id,
                &send_result,
                transaction_id.chain_id(),
            )
            .await
        }
        SupportedNamespaces::Solana => {
            solana_check_transaction(
                state.clone(),
                &project_id,
                &send_result,
                transaction_id.chain_id(),
            )
            .await
        }
        SupportedNamespaces::Tron => {
            tron_check_transaction(
                state.clone(),
                &project_id,
                &send_result,
                transaction_id.chain_id(),
            )
            .await
        }
    }?;

    let check_in = result.check_in;
    let txid = result.txid.clone();

    state.analytics.pos_check(PosCheckTxInfo::new(
        project_id.clone(),
        transaction_id.chain_id().to_string(),
        transaction_id.to_string(),
        send_result,
        &result.status,
        check_in,
        txid,
    ));

    Ok(result)
}
