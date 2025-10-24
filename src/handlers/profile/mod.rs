use {
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

pub mod address;
pub mod attributes;
pub mod lookup;
pub mod register;
pub mod reverse;
pub mod suggestions;

pub const UNIXTIMESTAMP_SYNC_THRESHOLD: u64 = 10;

/// Empty vector as an empty response
/// This is used to return an empty response when there are no results
pub const EMPTY_RESPONSE: Vec<String> = Vec::new();

/// Payload to register domain name that should be serialized to JSON
/// and passed to the RegisterRequest.message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterPayload {
    /// Name to register
    pub name: String,
    /// Attributes
    pub attributes: Option<HashMap<String, String>>,
    /// Unixtime
    pub timestamp: u64,
}

/// Payload to update name attributes that should be serialized to JSON and
/// signed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateAttributesPayload {
    /// Attributes
    pub attributes: HashMap<String, String>,
    /// Unixtime
    pub timestamp: u64,
}

/// Payload to update name address that should be serialized to JSON and signed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateAddressPayload {
    /// Coin type ENSIP-11
    pub coin_type: u32,
    /// Address
    pub address: String,
    /// Unixtime
    pub timestamp: u64,
}

/// Data structure representing a request to register a name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// Serialized JSON register payload
    pub message: String,
    /// Message signature
    pub signature: String,
    /// Coin type ENSIP-11
    pub coin_type: u32,
    /// Address
    pub address: String,
}

/// Forward and reverse lookup query parameters
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LookupQueryParams {
    /// Optional version parameter to support version-dependent responses
    pub api_version: Option<usize>,
    /// Request sender address for analytics
    pub sender: Option<String>,
}

/// Name suggestions query parameters
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionsParams {
    /// Optional zone to use for name suggestions
    pub zone: Option<String>,
}
