use {
    super::super::HANDLER_TASK_METRICS,
    crate::{
        error::RpcError, state::AppState, storage::irn::OperationType,
        utils::crypto::disassemble_caip10,
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListPermissionResponse {
    pci: Vec<String>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, address)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_list"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
) -> Result<Response, RpcError> {
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&address)?;

    // get all permission control identifiers for the address
    let irn_call_start = SystemTime::now();
    let pci = irn_client.hfields(address).await?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hfields);
    let response = ListPermissionResponse { pci };

    Ok(Json(response).into_response())
}
