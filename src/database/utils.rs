use std::collections::HashMap;

pub fn hashmap_to_hstore(hashmap: &HashMap<String, String>) -> String {
    hashmap
        .iter()
        .map(|(key, value)| format!("\"{}\" => \"{}\"", key, value))
        .collect::<Vec<_>>()
        .join(", ")
}
