use {
    super::{super::HANDLER_TASK_METRICS, GetPermissionsRequest, StoragePermissionsItem},
    crate::{
        error::RpcError, state::AppState, storage::irn::OperationType,
        utils::crypto::disassemble_caip10,
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde_json::json,
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    request: Path<GetPermissionsRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, request)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_create"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(request): Path<GetPermissionsRequest>,
) -> Result<Response, RpcError> {
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&request.address)?;

    let irn_call_start = SystemTime::now();
    let storage_permissions_item = irn_client
        .hget(request.address.clone(), request.pci.clone())
        .await?
        .ok_or_else(|| RpcError::PermissionNotFound(request.address.clone(), request.pci))?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hget);
    let storage_permissions_item =
        serde_json::from_str::<StoragePermissionsItem>(&storage_permissions_item)?;

    let response = json!({"context": storage_permissions_item.context});

    Ok(Json(response).into_response())
}
