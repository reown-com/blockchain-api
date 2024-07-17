use rand::{distributions::Alphanumeric, Rng};

pub mod build;
pub mod crypto;
pub mod network;
pub mod rate_limit;
pub mod suggestions;

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
