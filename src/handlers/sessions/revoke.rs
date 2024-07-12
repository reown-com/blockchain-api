use {
    super::{super::HANDLER_TASK_METRICS, PermissionRevokeRequest, StoragePermissionsItem},
    crate::{
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::crypto::{disassemble_caip10, verify_ecdsa_signature},
    },
    axum::{
        extract::{Path, State},
        response::Response,
        Json,
    },
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    Json(request_payload): Json<PermissionRevokeRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_revoke"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
    request_payload: PermissionRevokeRequest,
) -> Result<Response, RpcError> {
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&address)?;

    // Get the PCI object from the IRN
    let irn_call_start = SystemTime::now();
    let storage_permissions_item = irn_client
        .hget(address.clone(), request_payload.pci.clone())
        .await?
        .ok_or_else(|| RpcError::PermissionNotFound(request_payload.pci.clone()))?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hget.into());
    let storage_permissions_item =
        serde_json::from_str::<StoragePermissionsItem>(&storage_permissions_item)?;

    // Check the signature
    verify_ecdsa_signature(
        &request_payload.pci,
        &request_payload.signature,
        &storage_permissions_item.verification_key,
    )?;

    // Remove the session/permission item from the IRN
    let irn_call_start = SystemTime::now();
    irn_client.hdel(address, request_payload.pci).await?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hdel.into());

    Ok(Response::default())
}
