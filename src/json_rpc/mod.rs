#![warn(missing_docs)]

//! The crate exports common types used when interacting with messages between
//! clients.

use {
    derive_more::From,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
};

#[cfg(test)]
mod tests;

pub const JSON_RPC_VERSION_STR: &str = "2.0";

pub static JSON_RPC_VERSION: once_cell::sync::Lazy<Arc<str>> =
    once_cell::sync::Lazy::new(|| Arc::from(JSON_RPC_VERSION_STR));

/// Enum representing a JSON RPC Payload.
#[cfg(test)]
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
    pub id: serde_json::Value,
    /// The JSON RPC version.
    pub jsonrpc: Arc<str>,
    /// The RPC method.
    pub method: Arc<str>,
    /// The RPC params.
    pub params: T,
}

impl JsonRpcRequest {
    /// Create a new instance.
    pub fn new(id: serde_json::Value, method: Arc<str>) -> Self {
        Self {
            id,
            jsonrpc: JSON_RPC_VERSION.clone(),
            method,
            params: serde_json::Value::Null,
        }
    }
}

impl<T> JsonRpcRequest<T> {
    pub fn new_with_params(id: serde_json::Value, method: Arc<str>, params: T) -> Self {
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
    pub id: serde_json::Value,
    /// RPC version.
    pub jsonrpc: Arc<str>,
    /// The result for the message.
    pub result: T,
}

impl JsonRpcResult {
    pub fn new(id: serde_json::Value, result: serde_json::Value) -> Self {
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
    pub id: serde_json::Value,
    /// RPC version.
    pub jsonrpc: Arc<str>,
    /// The ErrorResponse corresponding to this message.
    pub error: ErrorResponse<T>,
}

impl<T: Serialize> JsonRpcError<T> {
    pub fn new(id: serde_json::Value, error: ErrorResponse<T>) -> Self {
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
