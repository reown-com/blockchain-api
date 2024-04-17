use {
    once_cell::sync::Lazy,
    regex::Regex,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

pub mod address;
pub mod attributes;
pub mod lookup;
pub mod register;
pub mod reverse;
pub mod suggestions;
pub mod utils;

/// List of allowed name zones
pub const ALLOWED_ZONES: [&str; 1] = ["wc.ink"];

pub const UNIXTIMESTAMP_SYNC_THRESHOLD: u64 = 10;

/// Attributes value max length
const ATTRIBUTES_VALUE_MAX_LENGTH: usize = 255;

/// List of supported attributes with the regex check pattern
static SUPPORTED_ATTRIBUTES: Lazy<HashMap<String, Regex>> = Lazy::new(|| {
    let mut map: HashMap<String, Regex> = HashMap::new();
    map.insert(
        "bio".into(),
        Regex::new(r"^[a-zA-Z0-9@:/._\-?&=+ ]+$").expect("Invalid regex for bio"),
    );
    map
});

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
