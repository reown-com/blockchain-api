use {
    super::{
        BuildPosTxsError, BuildTransactionParams, BuildTransactionResult, PaymentIntent,
        SupportedNamespaces, TransactionBuilder, TransactionRpc, ValidationError,
    },
    crate::{
        analytics::pos_info::{
            PosBuildTxInfo, PosBuildTxNew, PosBuildTxRequest, PosBuildTxResponse,
        },
        handlers::json_rpc::pos::{
            evm::EvmTransactionBuilder, solana::SolanaTransactionBuilder,
            tron::TronTransactionBuilder,
        },
        state::AppState,
        utils::crypto::Caip19Asset,
    },
    axum::extract::State,
    futures_util::future::try_join_all,
    std::{str::FromStr, sync::Arc},
};

async fn build_transaction_for_intent(
    state: State<Arc<AppState>>,
    project_id: String,
    intent: PaymentIntent,
) -> Result<TransactionRpc, BuildPosTxsError> {
    let asset = Caip19Asset::parse(&intent.asset)
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string())))?;

    let namespace = SupportedNamespaces::from_str(asset.chain_id().namespace())
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string())))?;

    match namespace {
        SupportedNamespaces::Eip155 => {
            let builder = EvmTransactionBuilder;
            builder.validate_and_build(state, project_id, intent).await
        }
        SupportedNamespaces::Solana => {
            let builder = SolanaTransactionBuilder;
            builder.validate_and_build(state, project_id, intent).await
        }
        SupportedNamespaces::Tron => {
            let builder = TronTransactionBuilder;
            builder.validate_and_build(state, project_id, intent).await
        }
    }
}

#[tracing::instrument(skip(state), level = "debug")]
pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    params: BuildTransactionParams,
) -> Result<BuildTransactionResult, BuildPosTxsError> {
    if params.payment_intents.is_empty() {
        return Err(BuildPosTxsError::Validation(
            ValidationError::InvalidRequest("No payment intents found".to_string()),
        ));
    }

    let capabilities_str = params.capabilities.as_ref().map(|v| {
        serde_json::to_string(v).unwrap_or_else(|e| {
            tracing::warn!(?e, "Failed to serialize capabilities for analytics");
            "<serde_error>".to_string()
        })
    });
    let intents = params.payment_intents.clone();

    let futures = params.payment_intents.into_iter().map(|intent| {
        let state = state.clone();
        let project_id = project_id.clone();
        async move { build_transaction_for_intent(state, project_id, intent).await }
    });

    let transactions = try_join_all(futures).await?;
    let response = BuildTransactionResult { transactions };

    for (intent, tx) in intents.iter().zip(response.transactions.iter()) {
        let tx_params_string = serde_json::to_string(&tx.params).unwrap_or_else(|e| {
            tracing::warn!(
                ?e,
                tx_id = tx.id,
                method = tx.method,
                "Failed to serialize tx params for analytics"
            );
            "<serde_error>".to_string()
        });
        let capabilities_string = capabilities_str.as_deref();
        state
            .analytics
            .pos_build(PosBuildTxInfo::new(PosBuildTxNew {
                project_id: &project_id,
                request: PosBuildTxRequest {
                    asset: &intent.asset,
                    amount: &intent.amount,
                    recipient: &intent.recipient,
                    sender: &intent.sender,
                    capabilities: capabilities_string,
                },
                response: PosBuildTxResponse {
                    tx_id: &tx.id,
                    tx_chain_id: &tx.chain_id,
                    tx_method: &tx.method,
                    tx_params: &tx_params_string,
                },
            }));
    }

    Ok(response)
}
