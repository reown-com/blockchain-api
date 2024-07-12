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
    p256::{
        ecdsa::{SigningKey, VerifyingKey},
        pkcs8::EncodePrivateKey,
    },
    rand_core::OsRng,
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    tracing::error,
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

    // Generate ECDSA key pair
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    let verifying_key_base64 = BASE64_STANDARD.encode(verifying_key.to_sec1_bytes());
    // Signing key as DER in sec1 format
    let signing_key_der_base64 = BASE64_STANDARD.encode(
        signing_key
            .to_pkcs8_der()
            .map_err(|e| {
                error!(
                    "Error during conversion signing key to pkcs8 DER format: {:?}",
                    e
                );
                RpcError::EcdsaError(e.to_string())
            })?
            .as_bytes(),
    );

    // Store the permission item in the IRN database
    let storage_permissions_item = StoragePermissionsItem {
        permissions: request_payload.permission,
        context: None,
        verification_key: verifying_key_base64,
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
        .add_irn_latency(irn_call_start, OperationType::Hset.into());

    let response = NewPermissionResponse {
        pci,
        key: signing_key_der_base64,
    };

    Ok(Json(response).into_response())
}
