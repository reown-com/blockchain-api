use {
    super::{super::HANDLER_TASK_METRICS, CoSignRequest, StoragePermissionsItem},
    crate::{
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::crypto::{
            abi_encode_two_bytes_arrays, call_get_signature, call_get_user_op_hash,
            disassemble_caip10, pack_signature, send_user_operation_to_bundler, to_eip191_message,
            CaipNamespaces, ChainId,
        },
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    base64::prelude::*,
    ethers::{
        core::k256::ecdsa::SigningKey,
        signers::LocalWallet,
        types::{H160, H256},
        utils::keccak256,
    },
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    tracing::info,
    wc::future::FutureExt,
};

const ENTRY_POINT_V07_CONTRACT_ADDRESS: &str = "0x0000000071727De22E5E9d8BAf0edAc6f37da032";

/// Co-sign response schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoSignResponse {
    user_operation_tx_hash: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    Json(request_payload): Json<CoSignRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_co_sign"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(caip10_address): Path<String>,
    request_payload: CoSignRequest,
) -> Result<Response, RpcError> {
    // Checking the CAIP-10 address format
    let (namespace, chain_id, address) = disassemble_caip10(&caip10_address)?;
    if namespace != CaipNamespaces::Eip155 {
        return Err(RpcError::UnsupportedNamespace(namespace));
    }

    // ChainID validation
    let chain_id_uint = chain_id
        .parse::<u64>()
        .map_err(|_| RpcError::InvalidChainIdFormat(chain_id.clone()))?;
    if !ChainId::is_supported(chain_id_uint) {
        return Err(RpcError::UnsupportedChain(chain_id.clone()));
    }

    let h160_address = address
        .parse::<H160>()
        .map_err(|_| RpcError::InvalidAddress)?;
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
    )
    .await?;
    let eip191_user_op_hash = to_eip191_message(&user_op_hash);

    // Get the PCI object from the IRN
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;
    let irn_call_start = SystemTime::now();
    let storage_permissions_item = irn_client
        .hget(caip10_address.clone(), request_payload.pci.clone())
        .await?
        .ok_or_else(|| RpcError::PermissionNotFound(request_payload.pci.clone()))?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hget);
    let storage_permissions_item =
        serde_json::from_str::<StoragePermissionsItem>(&storage_permissions_item)?;

    // Check and get the permission context if it's updated
    let permission_context_item = storage_permissions_item
        .context
        .clone()
        .ok_or_else(|| RpcError::PermissionContextNotUpdated(request_payload.pci.clone()))?;
    let permission_context = hex::decode(
        permission_context_item
            .context
            .permissions_context
            .clone()
            .trim_start_matches("0x"),
    )
    .map_err(|e| {
        RpcError::WrongHexFormat(format!(
            "error:{:?} permission_context:{}",
            e.to_string(),
            permission_context_item.context.permissions_context
        ))
    })?;

    // Sign the userOp hash with the permission signing key
    let signing_key_bytes = BASE64_STANDARD
        .decode(storage_permissions_item.signing_key)
        .map_err(|e| RpcError::WrongBase64Format(e.to_string()))?;
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

    // Get the Signature from the UserOpBuilder
    let user_op_builder_contract_address = permission_context_item
        .context
        .signer_data
        .user_op_builder
        .parse::<H160>()
        .map_err(|_| RpcError::InvalidAddress)?;
    let get_signature_result = call_get_signature(
        rpc_project_id,
        &chain_id_caip2,
        user_op_builder_contract_address,
        h160_address,
        user_op.clone(),
        permission_context.into(),
    )
    .await?;

    // Todo: remove this debug line before production stage
    info!("UserOpPacked final JSON: {:?}", serde_json::json!(user_op));

    // Update the userOp with the signature,
    // send the userOperation to the bundler and get the receipt
    user_op.signature = get_signature_result;
    let bundler_api_token =
        state
            .config
            .providers
            .bundler_token
            .clone()
            .ok_or(RpcError::InvalidConfiguration(
                "Missing bundler API token".to_string(),
            ))?;
    let user_operation_tx_hash = send_user_operation_to_bundler(
        &user_op,
        &chain_id,
        &bundler_api_token,
        ENTRY_POINT_V07_CONTRACT_ADDRESS,
        &state.http_client,
    )
    .await?;

    Ok(Json(CoSignResponse {
        user_operation_tx_hash,
    })
    .into_response())
}
