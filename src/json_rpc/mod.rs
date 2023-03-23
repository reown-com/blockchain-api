#![warn(missing_docs)]

//! The crate exports common types used when interacting with messages between
//! clients.

use {
    derive_more::{Display, From, Into},
    serde::{Deserialize, Serialize},
    serde_aux::prelude::deserialize_number_from_string,
    std::sync::Arc,
};

#[cfg(test)]
mod tests;

pub const JSON_RPC_VERSION: &str = "2.0";

/// Represents the message ID type.
#[derive(Copy, Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize, From, Into, Display)]
#[serde(transparent)]
pub struct MessageId(#[serde(deserialize_with = "deserialize_number_from_string")] u64);

/// Enum representing a JSON RPC Payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcPayload {
    /// A request to the server
    Request(JsonRpcRequest),
    /// A response to a request.
    Response(JsonRpcResponse),
}

/// Data structure representing a JSON RPC Request
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// ID this message corresponds to.
    pub id: MessageId,
    /// The JSON RPC version.
    pub jsonrpc: Arc<str>,
    /// The RPC method.
    pub method: Arc<str>,
}

impl JsonRpcRequest {
    /// Create a new instance.
    pub fn new(id: MessageId, method: Arc<str>) -> Self {
        Self {
            id,
            jsonrpc: JSON_RPC_VERSION.into(),
            method,
        }
    }
}

/// Enum representing a JSON RPC Response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse {
    /// A response with a result.
    Result(JsonRpcResult),
    /// A response depicting an error.
    Error(JsonRpcError),
}

/// Data structure representing a JSON RPC Result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcResult {
    /// ID this message corresponds to.
    pub id: MessageId,
    /// RPC version.
    pub jsonrpc: Arc<str>,
    /// The result for the message.
    pub result: serde_json::Value,
}

/// Data structure representing a JSON RPC Error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// ID this message corresponds to.
    pub id: MessageId,
    /// RPC version.
    pub jsonrpc: Arc<str>,
    /// The ErrorResponse corresponding to this message.
    pub error: ErrorResponse,
}

/// Data structure representing a ErrorResponse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: Arc<str>,
    /// Error data, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Arc<str>>,
}
