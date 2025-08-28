use {
    super::{
        BuildPosTxError, BuildTransactionParams, BuildTransactionResult, SupportedNamespaces,
        TransactionBuilder,
    },
    crate::{
        handlers::wallet::pos::evm::EvmTransactionBuilder, state::AppState,
        utils::crypto::Caip19Asset,
    },
    axum::extract::State,
    std::{str::FromStr, sync::Arc},
};

#[tracing::instrument(skip(_state), level = "debug")]
pub async fn handler(
    _state: State<Arc<AppState>>,
    project_id: String,
    params: BuildTransactionParams,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    let asset = Caip19Asset::parse(&params.asset)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid Asset: {e}")))?;

    let namespace = SupportedNamespaces::from_str(asset.chain_id().namespace())
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid namespace: {e}")))?;

    match namespace {
        SupportedNamespaces::Eip155 => {
            EvmTransactionBuilder
                .build(_state, project_id, params)
                .await
        }
    }
}
