use {
    crate::utils::crypto::UserOperation,
    alloy::primitives::Bytes,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    uuid::Uuid,
};

pub mod context;
pub mod cosign;
pub mod create;
pub mod get;
pub mod list;
pub mod revoke;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub project_id: String,
}

/// Payload to create a new permission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPermissionPayload {
    pub expiry: usize,
    pub signer: PermissionTypeData,
    pub permissions: Vec<PermissionTypeData>,
    pub policies: Vec<PermissionTypeData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionTypeData {
    pub r#type: String,
    pub data: Value,
}
// Payload to get permission by PCI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPermissionsRequest {
    address: String,
    pci: Uuid,
}

/// Permissions Context item schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivatePermissionPayload {
    pub pci: String,
    pub expiry: usize,
    pub signer: PermissionTypeData,
    pub permissions: Vec<PermissionTypeData>,
    pub policies: Vec<PermissionTypeData>,
    pub context: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionSubContextSignerData {
    user_op_builder: String,
}

/// Serialized permission item schema to store it in the IRN database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePermissionsItem {
    expiry: usize,
    created_at: usize,
    project_id: String,
    signer: PermissionTypeData,
    permissions: Vec<PermissionTypeData>,
    policies: Vec<PermissionTypeData>,
    context: Option<Bytes>,
    verification_key: String,
    signing_key: String,
}

/// Permission revoke request schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRevokeRequest {
    pci: String,
}

/// Co-sign request schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoSignRequest {
    pub pci: String,
    pub user_op: UserOperation,
}
