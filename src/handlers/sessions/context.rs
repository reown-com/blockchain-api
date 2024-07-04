use {
    super::{super::HANDLER_TASK_METRICS, PermissionContextItem, StoragePermissionsItem},
    crate::{
        error::RpcError,
        state::AppState,
        utils::crypto::{disassemble_caip10, json_canonicalize, verify_ecdsa_signature},
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    base64::prelude::*,
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    Json(request_payload): Json<PermissionContextItem>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_context_update"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
    request_payload: PermissionContextItem,
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
    state.metrics.add_irn_latency(irn_call_start, "hget".into());
    let mut storage_permissions_item =
        serde_json::from_str::<StoragePermissionsItem>(&storage_permissions_item)?;

    // Check the signature
    let json_canonicalized = json_canonicalize(serde_json::to_value(&request_payload.context)?)?;
    let keccak256_hash = BASE64_STANDARD.encode(ethers::core::utils::keccak256(json_canonicalized));
    verify_ecdsa_signature(
        &keccak256_hash,
        &request_payload.signature,
        &storage_permissions_item.verification_key,
    )?;

    // Update the context
    storage_permissions_item.context = Some(request_payload.clone());

    // Store it back to the IRN database
    let irn_call_start = SystemTime::now();
    irn_client
        .hset(
            address,
            request_payload.pci,
            serde_json::to_string(&storage_permissions_item)?.into(),
        )
        .await?;
    state.metrics.add_irn_latency(irn_call_start, "hset".into());

    Ok(Json(storage_permissions_item).into_response())
}
