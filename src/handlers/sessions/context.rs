use {
    super::{
        super::HANDLER_TASK_METRICS, ActivatePermissionPayload, QueryParams, StoragePermissionsItem,
    },
    crate::{
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::{crypto::disassemble_caip10, simple_request_json::SimpleRequestJson},
    },
    axum::{
        extract::{Path, Query, State},
        response::{IntoResponse, Response},
    },
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    query_params: Query<QueryParams>,
    SimpleRequestJson(request_payload): SimpleRequestJson<ActivatePermissionPayload>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_context_update"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
    query_params: Query<QueryParams>,
    request_payload: ActivatePermissionPayload,
) -> Result<Response, RpcError> {
    let project_id = query_params.project_id.clone();
    state.validate_project_access_and_quota(&project_id).await?;

    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&address)?;

    // Get the PCI object from the IRN
    let irn_call_start = SystemTime::now();
    let storage_permissions_item = irn_client
        .hget(address.clone(), request_payload.pci.clone())
        .await?
        .ok_or_else(|| {
            RpcError::PermissionNotFound(address.clone(), request_payload.pci.clone())
        })?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hget);
    let mut storage_permissions_item =
        serde_json::from_slice::<StoragePermissionsItem>(&storage_permissions_item)?;

    // Update the context
    storage_permissions_item.context = Some(request_payload.context);

    // Store it back to the IRN database
    let irn_call_start = SystemTime::now();
    irn_client
        .hset(
            address,
            request_payload.pci,
            serde_json::to_vec(&storage_permissions_item)?,
        )
        .await?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hset);

    Ok(().into_response())
}
