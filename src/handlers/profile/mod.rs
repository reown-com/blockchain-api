use {
    ethers::types::H160,
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    },
    tap::TapFallible,
};

pub mod lookup;
pub mod register;
pub mod reverse;

pub const UNIXTIMESTAMP_SYNC_THRESHOLD: u64 = 10;

/// Payload to register domain name that should be serialized to JSON
/// and passed to the RegisterRequest.message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterPayload {
    /// Name to register
    pub name: String,
    /// Coin type SLIP-44
    pub coin_type: u32,
    /// Chain ID for the EVM
    pub chain_id: u32,
    /// Address
    pub address: String,
    /// Attributes
    pub attributes: Option<HashMap<String, String>>,
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
    /// Coin type SLIP-44
    pub coin_type: u32,
    /// Address
    pub address: String,
}

#[tracing::instrument]
pub fn verify_message_signature(
    message: &str,
    signature: &str,
    owner: &H160,
) -> Result<bool, Box<dyn std::error::Error>> {
    let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    let message_hash = ethers::core::utils::keccak256(prefixed_message.clone());
    let message_hash = ethers::types::H256::from_slice(&message_hash);

    let sign = ethers::types::Signature::from_str(signature)?;
    match sign.verify(message_hash, *owner) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Check if the given unixtimestamp is within the threshold interval relative
/// to the current time
#[tracing::instrument(level = "debug")]
pub fn is_timestamp_within_interval(unix_timestamp: u64, threshold_interval: u64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .tap_err(|_| tracing::error!("SystemTime before UNIX EPOCH!"))
        .unwrap_or_default()
        .as_secs();

    unix_timestamp >= (now - threshold_interval) && unix_timestamp <= (now + threshold_interval)
}

#[cfg(test)]
mod tests {
    use {super::*, ethers::types::H160, std::str::FromStr};

    #[test]
    fn test_verify_message_signature_valid() {
        let message = "test message signature";
        let signature = "0x660739ee06920c5f55fbaf0da4f435faaa9c55e2c9da303c50c4b3865191d67e5002a0b10eb0f89bae66823f7f07415ea9d5bbb607ee61ac98b7f2a0a44fcb5c1b";
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap();

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_message_signature_json() {
        let message = r#"{\"test\":\"some my text\"}"#;
        let signature = "0x2fe0b640b4036c9c97911e6f22c72a2c934f1d67db02948055c0e0c84dbf4f2b33c2f8c4b000642735dbf5d1c96ba48ccd2a998324c9e4cb7bb776f0c95ee2fc1b";
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap();

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        println!("result: {:?}", result);
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_message_signature_invalid() {
        let message = "wrong message signature";
        let signature = "0x660739ee06920c5f55fbaf0da4f435faaa9c55e2c9da303c50c4b3865191d67e5002a0b10eb0f89bae66823f7f07415ea9d5bbb607ee61ac98b7f2a0a44fcb5c1b"; // The signature of the message
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap(); // The Ethereum address of the signer

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_is_timestamp_within_interval_valid() {
        let threshold_interval = 10;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .tap_err(|_| tracing::error!("SystemTime before UNIX EPOCH!"))
            .unwrap_or_default()
            .as_secs();
        assert!(is_timestamp_within_interval(now, threshold_interval));
    }

    #[test]
    fn test_verify_is_timestamp_within_interval_invalid() {
        let threshold_interval = 10;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .tap_err(|_| tracing::error!("SystemTime before UNIX EPOCH!"))
            .unwrap_or_default()
            .as_secs();
        // Upper bound reached
        assert!(!is_timestamp_within_interval(
            now + threshold_interval + 1,
            threshold_interval
        ));
        // Lower bound reached
        assert!(!is_timestamp_within_interval(
            now - threshold_interval - 1,
            threshold_interval
        ));
    }
}
