use {
    super::{super::HANDLER_TASK_METRICS, PermissionTypeData, QueryParams, StoragePermissionsItem},
    crate::{
        error::RpcError, state::AppState, storage::irn::OperationType,
        utils::crypto::disassemble_caip10,
    },
    alloy::primitives::Bytes,
    axum::{
        extract::{Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListPermissionResponse {
    pcis: Vec<Pci>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pci {
    pub project: ProjectItem,
    pub pci: String,
    pub expiration: usize,
    pub created_at: usize,
    pub permissions: Vec<PermissionTypeData>,
    pub policies: Vec<PermissionTypeData>,
    pub context: Option<Bytes>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

struct ProjectItem {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub icon_url: Option<String>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    query_params: Query<QueryParams>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, query_params)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_list"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
    query_params: Query<QueryParams>,
) -> Result<Response, RpcError> {
    let project_id = query_params.project_id.clone();
    state.validate_project_access_and_quota(&project_id).await?;

    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&address.clone())?;

    // get all permission control identifiers for the address
    let irn_call_start = SystemTime::now();
    let pcis = irn_client.hfields(address.clone()).await?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hfields);

    // get all permissions data for the pcis
    let mut result_pcis: Vec<Pci> = Vec::new();
    for pci in pcis {
        let irn_call_start = SystemTime::now();
        let storage_permissions_item = irn_client
            .hget(address.clone(), pci.clone())
            .await?
            .ok_or_else(|| RpcError::PermissionNotFound(address.clone(), pci.clone()))?;
        state
            .metrics
            .add_irn_latency(irn_call_start, OperationType::Hget);
        let storage_permissions_item =
            serde_json::from_str::<StoragePermissionsItem>(&storage_permissions_item)?;

        // Get project data
        let project = state
            .registry
            .project_data(&storage_permissions_item.project_id)
            .await?;

        result_pcis.push(Pci {
            project: ProjectItem {
                id: storage_permissions_item.project_id.clone(),
                name: project.project_data.name,
                url: None,
                icon_url: None,
            },
            pci,
            expiration: storage_permissions_item.expiration,
            created_at: storage_permissions_item.created_at,
            permissions: storage_permissions_item.permissions,
            policies: storage_permissions_item.policies,
            context: storage_permissions_item.context,
        });
    }

    Ok(Json(ListPermissionResponse { pcis: result_pcis }).into_response())
}
