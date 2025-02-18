use {
    super::{super::HANDLER_TASK_METRICS, CoSignRequest, StoragePermissionsItem},
    crate::{
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::{
            crypto::{
                abi_encode_two_bytes_arrays, call_get_user_op_hash, disassemble_caip10,
                is_address_valid, pack_signature, to_eip191_message, CaipNamespaces, ChainId,
                UserOperation,
            },
            permissions::{
                contract_call_permission_check, native_token_transfer_permission_check,
                ContractCallPermissionData, NativeTokenAllowancePermissionData, PermissionType,
            },
            sessions::extract_execution_batch_components,
        },
    },
    axum::{
        extract::{Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::{
        core::k256::ecdsa::SigningKey,
        signers::LocalWallet,
        types::{H160, H256},
        utils::keccak256,
    },
    serde::{Deserialize, Serialize},
    serde_json::json,
    std::{str::FromStr, sync::Arc, time::SystemTime},
    tracing::debug,
    wc::future::FutureExt,
};

const ENTRY_POINT_V07_CONTRACT_ADDRESS: &str = "0x0000000071727De22E5E9d8BAf0edAc6f37da032";

/// Co-sign response schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoSignResponse {
    user_operation_tx_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendUserOpRequest {
    pub chain_id: usize,
    pub user_op: UserOperation,
    pub permissions_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendUserOpResponse {
    pub receipt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoSignQueryParams {
    pub project_id: String,
    /// CoSigner version for testing purposes
    pub version: Option<u8>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    query_payload: Query<CoSignQueryParams>,
    Json(request_payload): Json<CoSignRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, request_payload, query_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_co_sign"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(caip10_address): Path<String>,
    request_payload: CoSignRequest,
    query_payload: Query<CoSignQueryParams>,
) -> Result<Response, RpcError> {
    let project_id = query_payload.project_id.clone();
    state.validate_project_access_and_quota(&project_id).await?;

    // Checking the CAIP-10 address format
    let (namespace, chain_id, address) = disassemble_caip10(&caip10_address)?;
    if namespace != CaipNamespaces::Eip155 {
        return Err(RpcError::UnsupportedNamespace(namespace));
    }
    if !is_address_valid(&address, &namespace) {
        return Err(RpcError::InvalidAddress);
    }

    // ChainID validation
    let chain_id_uint = chain_id
        .parse::<u64>()
        .map_err(|_| RpcError::InvalidChainIdFormat(chain_id.clone()))?;
    if !ChainId::is_supported(chain_id_uint) {
        return Err(RpcError::UnsupportedChain(chain_id.clone()));
    }

    let chain_id_caip2 = format!("{}:{}", namespace, chain_id);
    let mut user_op = request_payload.user_op.clone();

    // Project ID for internal json-rpc calls
    let rpc_project_id = state
        .config
        .server
        .testing_project_id
        .as_ref()
        .ok_or_else(|| {
            RpcError::InvalidConfiguration(
                "Missing testing project id in the configuration for the cosigner RPC calls"
                    .to_string(),
            )
        })?;

    // Get the userOp hash
    let contract_address = ENTRY_POINT_V07_CONTRACT_ADDRESS
        .parse::<H160>()
        .map_err(|_| RpcError::InvalidAddress)?;
    let user_op_hash = call_get_user_op_hash(
        rpc_project_id,
        &chain_id_caip2,
        contract_address,
        user_op.clone(),
        None,
    )
    .await?;
    let eip191_user_op_hash = to_eip191_message(&user_op_hash);

    // Get the PCI object from the IRN
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;
    let irn_call_start = SystemTime::now();
    let storage_permissions_item = irn_client
        .hget(caip10_address.clone(), request_payload.pci.clone())
        .await?
        .ok_or_else(|| {
            RpcError::PermissionNotFound(caip10_address.clone(), request_payload.pci.clone())
        })?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hget);
    let storage_permissions_item =
        serde_json::from_slice::<StoragePermissionsItem>(&storage_permissions_item)?;

    // Check if the permission is revoked
    if storage_permissions_item.revoked_at.is_some() {
        return Err(RpcError::RevokedPermission(request_payload.pci.clone()));
    }

    // Check if the permission is expired
    if storage_permissions_item.expiry
        < SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize
    {
        return Err(RpcError::PermissionExpired(request_payload.pci.clone()));
    }

    let call_data = request_payload.user_op.call_data.clone();
    // Extract the batch components
    let execution_batch = extract_execution_batch_components(&call_data)?;

    // Check the permissions length
    if storage_permissions_item.permissions.is_empty() {
        return Err(RpcError::CoSignerEmptyPermissions);
    }

    // Check permissions by types
    for permission in storage_permissions_item.permissions {
        match PermissionType::from_str(permission.r#type.as_str()) {
            Ok(permission_type) => match permission_type {
                PermissionType::ContractCall => {
                    debug!("Executing contract call permission check");
                    contract_call_permission_check(
                        execution_batch.clone(),
                        serde_json::from_value::<ContractCallPermissionData>(
                            permission.data.clone(),
                        )?,
                    )?;
                }
                PermissionType::NativeTokenRecurringAllowance => {
                    debug!("Executing native token transfer permission check");
                    native_token_transfer_permission_check(
                        execution_batch.clone(),
                        serde_json::from_value::<NativeTokenAllowancePermissionData>(
                            permission.data.clone(),
                        )?,
                    )?;
                }
            },
            Err(_) => {
                return Err(RpcError::CosignerUnsupportedPermission(permission.r#type));
            }
        }
    }

    // Check and get the permission context if it's updated
    let _permission_context = storage_permissions_item
        .context
        .clone()
        .ok_or_else(|| RpcError::PermissionContextNotUpdated(request_payload.pci.clone()))?;

    // Sign the userOp hash with the permission signing key
    let signing_key_bytes = hex::decode(storage_permissions_item.signing_key)
        .map_err(|e| RpcError::WrongHexFormat(e.to_string()))?;
    let signer = SigningKey::from_bytes(signing_key_bytes.as_slice().into())
        .map_err(|e| RpcError::KeyFormatError(e.to_string()))?;

    // Create a LocalWallet for signing and signing the hashed message
    let wallet = LocalWallet::from(signer);
    let signature = wallet
        .sign_hash(H256::from(&keccak256(eip191_user_op_hash.clone())))
        .unwrap();
    let packed_signature = pack_signature(&signature);

    // ABI encode the signatures
    let concatenated_signature = abi_encode_two_bytes_arrays(&packed_signature, &user_op.signature);

    // Update the userOp with the signature
    user_op.signature = concatenated_signature;

    Ok(Json(json!({
        "signature": format!("0x{}", hex::encode(user_op.signature)),
    }))
    .into_response())
}
