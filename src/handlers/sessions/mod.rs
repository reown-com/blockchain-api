use serde::{Deserialize, Serialize};

pub mod create;
pub mod get;
pub mod list;

/// Payload to create a new permission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewPermissionPayload {
    pub permissions: PermissionItem,
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

/// Serialized permission item schema to store it in the IRN database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePermissionsItem {
    permissions: PermissionItem,
    verification_key: String,
}
