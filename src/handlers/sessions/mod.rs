use serde::{Deserialize, Serialize};

pub mod context;
pub mod create;
pub mod get;
pub mod list;
pub mod revoke;

/// Payload to create a new permission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPermissionPayload {
    pub permission: PermissionItem,
}

// Payload to get permission by PCI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPermissionsRequest {
    address: String,
    pci: String,
}

/// Permission item schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionItem {
    permission_type: String,
    data: String,
    required: bool,
    on_chain_validated: bool,
}

/// Permissions Context item schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionContextItem {
    pci: String,
    context: PermissionSubContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionSubContext {
    signer: PermissionContextSigner,
    expiry: usize,
    signer_data: PermissionSubContextSignerData,
    factory: String,
    factory_data: String,
    permissions_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionContextSigner {
    r#type: String,
    data: PermissionContextSignerData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionContextSignerData {
    ids: Vec<String>,
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
    permissions: PermissionItem,
    context: Option<PermissionContextItem>,
    verification_key: String,
    signing_key: String,
}

/// Permission revoke request schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRevokeRequest {
    pci: String,
}
