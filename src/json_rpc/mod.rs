#![warn(missing_docs)]

//! The crate exports common types used when interacting with messages between
//! clients.

use {
    derive_more::{Display, From, Into},
    serde::{Deserialize, Serialize},
    serde_aux::prelude::deserialize_string_from_number,
    std::sync::Arc,
};

#[cfg(test)]
mod tests;

pub const JSON_RPC_VERSION_STR: &str = "2.0";

pub static JSON_RPC_VERSION: once_cell::sync::Lazy<Arc<str>> =
    once_cell::sync::Lazy::new(|| Arc::from(JSON_RPC_VERSION_STR));

/// Represents the message ID type.
#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize, Deserialize, From, Into, Display)]
#[serde(transparent)]
pub struct MessageId(#[serde(deserialize_with = "deserialize_string_from_number")] String);

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
pub struct JsonRpcRequest<T = serde_json::Value> {
    /// ID this message corresponds to.
    pub id: MessageId,
    /// The JSON RPC version.
    pub jsonrpc: Arc<str>,
    /// The RPC method.
    pub method: Arc<str>,
    /// The RPC params.
    pub params: T,
}

impl JsonRpcRequest {
    /// Create a new instance.
    pub fn new(id: MessageId, method: Arc<str>) -> Self {
        Self {
            id,
            jsonrpc: JSON_RPC_VERSION.clone(),
            method,
            params: serde_json::Value::Null,
        }
    }
}

impl<T> JsonRpcRequest<T> {
    pub fn new_with_params(id: MessageId, method: Arc<str>, params: T) -> Self {
        Self {
            id,
            jsonrpc: JSON_RPC_VERSION.clone(),
            method,
            params,
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
pub struct JsonRpcResult<T = serde_json::Value> {
    /// ID this message corresponds to.
    pub id: MessageId,
    /// RPC version.
    pub jsonrpc: Arc<str>,
    /// The result for the message.
    pub result: T,
}

impl JsonRpcResult {
    pub fn new(id: MessageId, result: serde_json::Value) -> Self {
        Self {
            id,
            jsonrpc: JSON_RPC_VERSION.clone(),
            result,
        }
    }
}

/// Data structure representing a JSON RPC Error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError<T: Serialize = Option<Arc<str>>> {
    /// ID this message corresponds to.
    pub id: MessageId,
    /// RPC version.
    pub jsonrpc: Arc<str>,
    /// The ErrorResponse corresponding to this message.
    pub error: ErrorResponse<T>,
}

impl<T: Serialize> JsonRpcError<T> {
    pub fn new(id: MessageId, error: ErrorResponse<T>) -> Self {
        Self {
            id,
            jsonrpc: JSON_RPC_VERSION.clone(),
            error,
        }
    }
}

/// Data structure representing a ErrorResponse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse<T: Serialize> {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: Arc<str>,
    /// Error data, if any.
    pub data: T,
}
