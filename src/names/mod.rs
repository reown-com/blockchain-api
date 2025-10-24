use {once_cell::sync::Lazy, regex::Regex, serde::Deserialize, std::collections::HashMap};

pub mod suggestions;
pub mod utils;

/// Attributes value max length
pub const ATTRIBUTES_VALUE_MAX_LENGTH: usize = 255;

/// List of supported attributes with the regex check pattern
pub static SUPPORTED_ATTRIBUTES: Lazy<HashMap<String, Regex>> = Lazy::new(|| {
    let mut map: HashMap<String, Regex> = HashMap::new();
    map.insert(
        "bio".into(),
        Regex::new(r"^[a-zA-Z0-9@:/._\-?&=+ ]+$").expect("Invalid regex for bio"),
    );
    map
});

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub allowed_zones: Option<Vec<String>>,
}
