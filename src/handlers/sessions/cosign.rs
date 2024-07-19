use {
    super::{super::HANDLER_TASK_METRICS, CoSignRequest, StoragePermissionsItem},
    crate::{
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::crypto::{
            abi_encode_two_bytes_arrays, call_get_signature, call_get_user_op_hash,
            disassemble_caip10, send_user_operation_to_bundler, CaipNamespaces,
        },
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    base64::prelude::*,
    ethers::{
        core::k256::{
            ecdsa::{signature::Signer, Signature, SigningKey},
            pkcs8::DecodePrivateKey,
        },
        types::{Bytes, H160},
    },
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

/// Co-sign response schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoSignResponse {
    receipt: String,
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
    Path(address): Path<String>,
    request_payload: CoSignRequest,
) -> Result<Response, RpcError> {
    // Checking the CAIP-10 address format
    let (namespace, chain_id, address) = disassemble_caip10(&address)?;
    if namespace != CaipNamespaces::Eip155 {
        return Err(RpcError::UnsupportedNamespace(namespace));
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
                "Missing testing project id in the configuration for the balance RPC lookups"
                    .to_string(),
            )
        })?;

    // Get the userOp hash
    // Entrypoint v07 contract address
    let contract_address = "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
        .parse::<H160>()
        .map_err(|_| RpcError::InvalidAddress)?;
    let user_op_hash = call_get_user_op_hash(
        rpc_project_id,
        &chain_id_caip2,
        contract_address,
        user_op.clone(),
    )
    .await?;

    // Get the PCI object from the IRN
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;
    let irn_call_start = SystemTime::now();
    let storage_permissions_item = irn_client
        .hget(address.clone(), request_payload.pci.clone())
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
    let permission_context = hex::decode(permission_context_item.context.permissions_context)
        .map_err(|e| RpcError::WrongHexFormat(e.to_string()))?;

    // Sign the userOp hash with the permission signing key
    let signing_key_bytes = BASE64_STANDARD
        .decode(storage_permissions_item.signing_key)
        .map_err(|e| RpcError::WrongBase64Format(e.to_string()))?;
    let signer = SigningKey::from_pkcs8_der(&signing_key_bytes)?;
    let signature: Signature = signer.sign(&user_op_hash);

    // ABI encode the signatures
    let concatenated_signature = abi_encode_two_bytes_arrays(
        &Bytes::from(signature.to_der().as_bytes().to_vec()),
        &user_op.signature,
    );

    // Update the userOp with the signature
    user_op.signature = concatenated_signature;

    // Get the Signature
    // UserOpBuilder contract address
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

    // Update the userOp with the signature
    user_op.signature = get_signature_result;

    // Using the Biconomy bundler to send the userOperation
    let entry_point = "0x5ff137d4b0fdcd49dca30c7cf57e578a026d2789";
    let simulation_type = "validation_and_execution";
    let bundler_url = format!(
        "https://bundler.biconomy.io/api/v2/{}/{}",
        chain_id,
        state
            .config
            .providers
            .biconomy_bundler_token
            .clone()
            .ok_or(RpcError::InvalidConfiguration(
                "Missing biconomy bundler token".to_string()
            ))?
    );

    // Send the userOperation to the bundler and get the receipt
    let receipt = send_user_operation_to_bundler(
        &user_op,
        &bundler_url,
        entry_point,
        simulation_type,
        &state.http_client,
    )
    .await?;

    Ok(Json(receipt).into_response())
}
