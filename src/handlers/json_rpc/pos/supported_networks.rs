use {
    super::{
        evm::get_namespace_info as evm_get_namespace_info,
        solana::get_namespace_info as solana_get_namespace_info,
        tron::get_namespace_info as tron_get_namespace_info,
        SupportedNetworksError,
        SupportedNetworksResult,
    },
    crate::state::AppState,
    axum::extract::State,
    std::sync::Arc,
};

pub async fn handler(
    _state: State<Arc<AppState>>,
    _project_id: String,
) -> Result<SupportedNetworksResult, SupportedNetworksError> {
    Ok(SupportedNetworksResult {
        namespaces: vec![
            evm_get_namespace_info(),
            solana_get_namespace_info(),
            tron_get_namespace_info(),
        ],
    })
}
