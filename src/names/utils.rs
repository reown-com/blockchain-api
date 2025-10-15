use {
    crate::database::helpers::get_name,
    once_cell::sync::Lazy,
    regex::Regex,
    sqlx::{Error as SqlxError, PgPool},
    std::{
        collections::HashMap,
        time::{SystemTime, UNIX_EPOCH},
    },
    tap::TapFallible,
    tracing::error,
};

static DOMAIN_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9.-]+$").expect("Failed to initialize regexp for the domain format")
});

const NAME_MIN_LENGTH: usize = 3;
const NAME_MAX_LENGTH: usize = 64;

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
pub fn is_name_in_allowed_zones(name: &str, allowed_zones: Vec<String>) -> bool {
    let name_parts: Vec<&str> = name.split('.').collect();
    if name_parts.len() != 3 {
        return false;
    }
    let tld = format!("{}.{}", name_parts[1], name_parts[2]);
    allowed_zones.contains(&tld)
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
    use super::{
        super::{ATTRIBUTES_VALUE_MAX_LENGTH, SUPPORTED_ATTRIBUTES},
        *,
    };

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
        let allowed_zones = vec!["eth.link".to_string(), "ens.domains".to_string()];

        let mut valid_name = "test.eth.link";
        assert!(is_name_in_allowed_zones(valid_name, allowed_zones.clone()));

        valid_name = "some.ens.domains";
        assert!(is_name_in_allowed_zones(valid_name, allowed_zones.clone()));

        let mut invalid_name = "test.com";
        assert!(!is_name_in_allowed_zones(
            invalid_name,
            allowed_zones.clone()
        ));

        invalid_name = "eth.link";
        assert!(!is_name_in_allowed_zones(
            invalid_name,
            allowed_zones.clone()
        ));

        invalid_name = "test.some.link";
        assert!(!is_name_in_allowed_zones(invalid_name, allowed_zones));
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
