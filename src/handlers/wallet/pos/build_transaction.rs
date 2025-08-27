use {
    super::{BuildPosTxError, BuildTransactionParams, BuildTransactionResult, TransactionBuilder},
    crate::{
        handlers::wallet::pos::evm::EvmTransactionBuilder, state::AppState,
        utils::crypto::Caip19Asset,
    },
    axum::extract::State,
    std::sync::Arc,
};

#[tracing::instrument(skip(_state), level = "debug")]
pub async fn handler(
    _state: State<Arc<AppState>>,
    project_id: String,
    params: BuildTransactionParams,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    let asset = Caip19Asset::parse(&params.asset)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid Asset: {e}")))?;

    match asset.chain_id().namespace() {
        "eip155" => {
            EvmTransactionBuilder
                .build(_state, project_id, params)
                .await
        }
        ns => Err(BuildPosTxError::Validation(format!(
            "Unsupported namespace: {ns}"
        ))),
    }
}
