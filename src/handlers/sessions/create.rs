use {
    super::{
        super::HANDLER_TASK_METRICS,
        NewPermissionPayload,
        QueryParams,
        StoragePermissionsItem,
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
        Json,
    },
    ethers::core::k256::ecdsa::{SigningKey, VerifyingKey},
    rand_core::OsRng,
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPermissionResponse {
    pci: String,
    key: KeyItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
    Secp256k1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyItem {
    pub r#type: KeyType,
    pub public_key: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    query_params: Query<QueryParams>,
    SimpleRequestJson(request_payload): SimpleRequestJson<NewPermissionPayload>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_create"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
    query_params: Query<QueryParams>,
    request_payload: NewPermissionPayload,
) -> Result<Response, RpcError> {
    let project_id = query_params.project_id.clone();
    state
        .validate_project_access_and_quota(&project_id.clone())
        .await?;

    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&address)?;

    // Generate a unique permission control identifier
    let pci = uuid::Uuid::new_v4().to_string();

    // Generate a secp256k1 keys and export to DER Base64 and Hex formats
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    let private_key_der = signing_key.to_bytes().to_vec();
    let private_key_der_hex = hex::encode(private_key_der);
    let public_key_der = verifying_key.to_encoded_point(false).as_bytes().to_vec();
    let public_key_der_hex = hex::encode(&public_key_der);

    // Store the permission item in the IRN database
    let storage_permissions_item = StoragePermissionsItem {
        // Storing the PCI inside of the item along with the item field
        // as a temporary hotfix solution for the WCN/IRN field wrong naming
        pci: pci.clone(),
        expiry: request_payload.expiry,
        created_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::new(0, 0))
            .as_secs() as usize,
        project_id,
        signer: request_payload.signer,
        permissions: request_payload.permissions,
        policies: request_payload.policies,
        context: None,
        verification_key: public_key_der_hex.clone(),
        signing_key: private_key_der_hex.clone(),
        revoked_at: None,
    };

    let irn_call_start = SystemTime::now();
    irn_client
        .hset(
            address.clone(),
            pci.clone(),
            serde_json::to_vec(&storage_permissions_item)?,
        )
        .await?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hset);

    // Format public key based on API version
    let public_key = match query_params.api_version {
        Some(2) => {
            // v2: Direct hex string with 0x prefix
            format!("0x{public_key_der_hex}")
        }
        _ => {
            // v1 (default): ASCII-hex encoded with 0x prefix
            format!("0x{}", hex::encode(public_key_der_hex))
        }
    };

    let response = NewPermissionResponse {
        pci: pci.clone(),
        key: KeyItem {
            r#type: KeyType::Secp256k1,
            public_key,
        },
    };

    // TODO: remove this debuging log
    print!(
        "New permission created with PCI: {:?} for address: {:?}",
        pci, address
    );

    Ok(Json(response).into_response())
}
