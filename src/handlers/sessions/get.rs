use {
    super::{
        super::HANDLER_TASK_METRICS, GetPermissionsRequest, PermissionItem, StoragePermissionsItem,
    },
    crate::{error::RpcError, state::AppState},
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    std::sync::Arc,
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

    let storage_permissions_item = match irn_client
        .hget(request.address.clone(), request.pci.clone())
        .await?
    {
        Some(storage_permissions_item) => storage_permissions_item,
        None => return Err(RpcError::PermissionNotFound(request.pci)),
    };
    let storage_permissions_item: StoragePermissionsItem =
        serde_json::from_str(&storage_permissions_item)?;

    let response = PermissionItem {
        permission_type: storage_permissions_item.permissions.permission_type,
        data: storage_permissions_item.permissions.data,
        required: storage_permissions_item.permissions.required,
        on_chain_validated: storage_permissions_item.permissions.on_chain_validated,
    };

    Ok(Json(response).into_response())
}
