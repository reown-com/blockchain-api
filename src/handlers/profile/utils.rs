use {
    crate::database::helpers::get_name,
    ethers::types::H160,
    once_cell::sync::Lazy,
    regex::Regex,
    sqlx::{Error as SqlxError, PgPool},
    std::{
        collections::HashMap,
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    },
    tap::TapFallible,
    tracing::error,
};

static DOMAIN_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9.-]+$").expect("Failed to initialize regexp for the domain format")
});

const NAME_MIN_LENGTH: usize = 3;
const NAME_MAX_LENGTH: usize = 64;

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

/// Check if the given attributes map contains only supported attributes
/// in the given format and length
pub fn check_attributes(
    attributes_map: &HashMap<String, String>,
    keys_allowed: &HashMap<String, Regex>,
    max_length: usize,
) -> bool {
    attributes_map.iter().all(|(key, value)| {
        if !keys_allowed.contains_key(key) {
            return false;
        }
        if value.is_empty() || value.len() > max_length {
            return false;
        }
        keys_allowed[key].is_match(value)
    })
}

/// Check if the given name is in the allowed zones
pub fn is_name_in_allowed_zones(name: &str, allowed_zones: &[&str]) -> bool {
    let name_parts: Vec<&str> = name.split('.').collect();
    if name_parts.len() != 3 {
        return false;
    }
    let tld = format!("{}.{}", name_parts[1], name_parts[2]);
    allowed_zones.contains(&tld.as_str())
}

/// Check if the given name is in the correct format
pub fn is_name_format_correct(name: &str) -> bool {
    DOMAIN_FORMAT_REGEX.is_match(name)
}

/// Check the given name length
pub fn is_name_length_correct(name: &str) -> bool {
    let name_parts: Vec<&str> = name.split('.').collect();
    if name_parts.len() != 3 {
        return false;
    }
    name_parts[0].len() >= NAME_MIN_LENGTH && name_parts[0].len() <= NAME_MAX_LENGTH
}

#[tracing::instrument(skip(postgres), level = "debug")]
pub async fn is_name_registered(name: String, postgres: &PgPool) -> bool {
    match get_name(name, postgres).await {
        Ok(_) => true,
        Err(e) => match e {
            SqlxError::RowNotFound => false,
            _ => {
                error!("Failed to lookup name: {}", e);
                false
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::handlers::profile::{ATTRIBUTES_VALUE_MAX_LENGTH, SUPPORTED_ATTRIBUTES},
        ethers::types::H160,
        std::str::FromStr,
    };

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

    #[test]
    fn test_check_attributes() {
        let valid_map: HashMap<String, String> = HashMap::from([("bio".into(), "Test bio".into())]);
        let invalid_key_map: HashMap<String, String> = HashMap::from([
            ("some_key".into(), "some text".into()),
            ("bio".into(), "Some bio".into()),
        ]);
        let invalid_character_map: HashMap<String, String> =
            HashMap::from([("bio".into(), "Bio *>".into())]);

        // Valid
        assert!(check_attributes(
            &valid_map,
            &SUPPORTED_ATTRIBUTES,
            ATTRIBUTES_VALUE_MAX_LENGTH,
        ));
        // Invalid attributes key
        assert!(!check_attributes(
            &invalid_key_map,
            &SUPPORTED_ATTRIBUTES,
            ATTRIBUTES_VALUE_MAX_LENGTH,
        ));
        // Invalid value length
        assert!(!check_attributes(&valid_map, &SUPPORTED_ATTRIBUTES, 4,));
        // Invalid characters
        assert!(!check_attributes(
            &invalid_character_map,
            &SUPPORTED_ATTRIBUTES,
            ATTRIBUTES_VALUE_MAX_LENGTH,
        ));
    }

    #[test]
    fn test_is_name_in_allowed_zones() {
        let allowed_zones = ["eth.link", "ens.domains"];

        let mut valid_name = "test.eth.link";
        assert!(is_name_in_allowed_zones(valid_name, &allowed_zones));

        valid_name = "some.ens.domains";
        assert!(is_name_in_allowed_zones(valid_name, &allowed_zones));

        let mut invalid_name = "test.com";
        assert!(!is_name_in_allowed_zones(invalid_name, &allowed_zones));

        invalid_name = "eth.link";
        assert!(!is_name_in_allowed_zones(invalid_name, &allowed_zones));

        invalid_name = "test.some.link";
        assert!(!is_name_in_allowed_zones(invalid_name, &allowed_zones));
    }

    #[test]
    fn test_is_name_format_correct() {
        let valid_name = "test.eth.link";
        assert!(is_name_format_correct(valid_name));

        let invalid_name = "test*.eth.link";
        assert!(!is_name_format_correct(invalid_name));
    }

    #[test]
    fn test_is_name_length_correct() {
        let name = "a".repeat(NAME_MIN_LENGTH) + ".test.eth";
        assert!(is_name_length_correct(&name));

        let name = "a".repeat(NAME_MAX_LENGTH) + ".test.eth";
        assert!(is_name_length_correct(&name));

        let name = "a".repeat(NAME_MIN_LENGTH - 1) + ".test.eth";
        assert!(!is_name_length_correct(&name));

        let name = "a".repeat(NAME_MAX_LENGTH + 1) + ".test.eth";
        assert!(!is_name_length_correct(&name));
    }
}
