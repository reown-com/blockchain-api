use {
    super::{super::HANDLER_TASK_METRICS, NewPermissionPayload, StoragePermissionsItem},
    crate::{
        error::RpcError, state::AppState, storage::irn::OperationType,
        utils::crypto::disassemble_caip10,
    },
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Response},
        Json,
    },
    base64::prelude::*,
    ethers::core::k256::ecdsa::{SigningKey, VerifyingKey},
    rand_core::OsRng,
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    wc::future::FutureExt,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPermissionResponse {
    pci: String,
    key: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    Json(request_payload): Json<NewPermissionPayload>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_create"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(address): Path<String>,
    request_payload: NewPermissionPayload,
) -> Result<Response, RpcError> {
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;

    // Checking the CAIP-10 address format
    disassemble_caip10(&address)?;

    // Generate a unique permission control identifier
    let pci = uuid::Uuid::new_v4().to_string();

    // Generate a secp256k1 keys and export to DER Base64 format
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    let private_key_der = signing_key.to_bytes().to_vec();
    let private_key_der_base64 = BASE64_STANDARD.encode(private_key_der);
    let public_key_der = verifying_key.to_encoded_point(false).as_bytes().to_vec();
    let public_key_der_base64 = BASE64_STANDARD.encode(&public_key_der);

    // Store the permission item in the IRN database
    let storage_permissions_item = StoragePermissionsItem {
        permissions: request_payload.permission,
        context: None,
        verification_key: public_key_der_base64.clone(),
        signing_key: private_key_der_base64,
    };

    let irn_call_start = SystemTime::now();
    irn_client
        .hset(
            address.clone(),
            pci.clone(),
            serde_json::to_string(&storage_permissions_item)?.into(),
        )
        .await?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hset);

    let response = NewPermissionResponse {
        pci: pci.clone(),
        key: public_key_der_base64,
    };

    // TODO: remove this debuging log
    print!(
        "New permission created with PCI: {:?} for address: {:?}",
        pci, address
    );

    Ok(Json(response).into_response())
}
