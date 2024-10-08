use {
    super::{
        super::HANDLER_TASK_METRICS, GetPermissionsRequest, QueryParams, StoragePermissionsItem,
    },
    crate::{
        error::RpcError,
        metrics::Metrics,
        state::AppState,
        storage::{
            error::StorageError,
            irn::{Irn, OperationType},
        },
        utils::crypto::disassemble_caip10,
    },
    alloy::primitives::Bytes,
    axum::{
        extract::{Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde_json::json,
    std::{sync::Arc, time::SystemTime},
    uuid::Uuid,
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    request: Path<GetPermissionsRequest>,
    query_params: Query<QueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, request, query_params)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_create"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(request): Path<GetPermissionsRequest>,
    query_params: Query<QueryParams>,
) -> Result<Response, RpcError> {
    let project_id = query_params.project_id.clone();
    state.validate_project_access_and_quota(&project_id).await?;

    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&request.address)?;

    let context = get_session_context(
        request.address.clone(),
        request.pci,
        irn_client,
        &state.metrics,
    )
    .await
    .map_err(|e| match e {
        GetSessionContextError::PermissionNotFound(address, pci) => {
            RpcError::PermissionNotFound(address.to_string(), pci.to_string())
        }
        GetSessionContextError::InternalGetSessionContextError(e) => {
            RpcError::InternalGetSessionContextError(e)
        }
    })?;

    let response = json!({"context": context});

    Ok(Json(response).into_response())
}

#[derive(thiserror::Error, Debug)]
pub enum GetSessionContextError {
    #[error("Permission not found for address {0} and PCI {1}")]
    PermissionNotFound(String, Uuid),

    #[error("Internal error: {0}")]
    InternalGetSessionContextError(InternalGetSessionContextError),
}

#[derive(thiserror::Error, Debug)]
pub enum InternalGetSessionContextError {
    #[error("Storage: {0}")]
    Storage(StorageError),

    #[error("Deserializing: {0}")]
    Deserializing(serde_json::Error),
}

pub async fn get_session_context(
    address: String,
    pci: Uuid,
    irn_client: &Irn,
    metrics: &Metrics,
) -> Result<Option<Bytes>, GetSessionContextError> {
    let irn_call_start = SystemTime::now();
    let storage_permissions_item = irn_client
        .hget(address.clone(), pci.to_string())
        .await
        .map_err(|e| {
            GetSessionContextError::InternalGetSessionContextError(
                InternalGetSessionContextError::Storage(e),
            )
        })?
        .ok_or_else(|| GetSessionContextError::PermissionNotFound(address, pci))?;
    metrics.add_irn_latency(irn_call_start, OperationType::Hget);

    let storage_permissions_item =
        serde_json::from_str::<StoragePermissionsItem>(&storage_permissions_item).map_err(|e| {
            GetSessionContextError::InternalGetSessionContextError(
                InternalGetSessionContextError::Deserializing(e),
            )
        })?;
    Ok(storage_permissions_item.context)
}
