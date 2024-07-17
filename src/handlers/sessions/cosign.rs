use {
    super::{super::HANDLER_TASK_METRICS, CoSignRequest, StoragePermissionsItem},
    crate::{
        analytics::MessageSource,
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::crypto::{
            disassemble_caip10, send_user_operation_to_bundler, verify_message_signature,
            CaipNamespaces,
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
        utils::keccak256,
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
    let chain_id_caip2 = format!("{}:{}", namespace, chain_id);

    // Verify the user signature
    let mut user_op = request_payload.user_op.clone();
    let user_op_hash = keccak256(
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            user_op.sender,
            user_op.nonce,
            user_op.init_code,
            user_op.call_data,
            user_op.call_data,
            user_op.call_gas_limit,
            user_op.verification_gas_limit,
            user_op.pre_verification_gas,
            user_op.max_fee_per_gas,
            user_op.max_priority_fee_per_gas,
            user_op.paymaster_and_data
        )
        .as_bytes(),
    );
    let user_op_hash = hex::encode(user_op_hash);
    let rpc_project_id = state
        .config
        .server
        .testing_project_id
        .as_ref()
        .ok_or_else(|| {
            RpcError::InvalidConfiguration(
                "Missing testing project id in the configuration for eip1271 lookups".to_string(),
            )
        })?;
    let sinature_check = verify_message_signature(
        &user_op_hash,
        &request_payload.user_op.signature.clone(),
        &address,
        &chain_id_caip2,
        rpc_project_id,
        MessageSource::SessionCoSignSigValidate,
    )
    .await?;
    if !sinature_check {
        return Err(RpcError::SignatureValidationError(
            "Signature verification error".into(),
        ));
    }

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

    // Sign the user operation with the permission signing key
    let signing_key_bytes = BASE64_STANDARD
        .decode(storage_permissions_item.signing_key)
        .map_err(|e| RpcError::WrongBase64Format(e.to_string()))?;
    let signer = SigningKey::from_pkcs8_der(&signing_key_bytes)?;
    let signature: Signature = signer.sign(user_op_hash.as_bytes());
    let signature_hex = hex::encode(signature.to_der().as_bytes());

    // Concat permission and user signature, update the userOperation signature
    let concatenated_signature = format!("{}{}", signature_hex, request_payload.user_op.signature);
    user_op.signature = concatenated_signature;

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
