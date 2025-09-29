use rand::{distributions::Alphanumeric, Rng};

pub mod batch_json_rpc_request;
pub mod build;
pub mod cors;
pub mod crypto;
pub mod erc4337;
pub mod erc7677;
pub mod json_rpc_cache;
pub mod network;
pub mod permissions;
pub mod rate_limit;
pub mod sessions;
pub mod simple_request_json;
pub mod token_amount;
pub mod validators;

pub fn generate_random_string(len: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .filter_map(|b| {
            let c = b as char;
            if c.is_ascii_alphanumeric() || c.is_ascii_digit() {
                Some(c)
            } else {
                None
            }
        })
        .take(len)
        .collect()
}

pub fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => {
            // to_uppercase() returns an iterator because some characters can map to
            // multiple chars
            first.to_uppercase().collect::<String>() + c.as_str()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first_letter() {
        let input = "";
        let expected = "";
        assert_eq!(capitalize_first_letter(input), expected);

        let input = "rust";
        let expected = "Rust";
        assert_eq!(capitalize_first_letter(input), expected);

        let input = "rust world";
        let expected = "Rust world";
        assert_eq!(capitalize_first_letter(input), expected);
    }
}
