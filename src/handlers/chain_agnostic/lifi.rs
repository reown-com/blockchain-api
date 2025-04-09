use crate::error::RpcError;
use yttrium::chain_abstraction::solana;

pub fn caip2_to_lifi_chain_id(caip2: &str) -> Result<&str, RpcError> {
    match caip2 {
        solana::SOLANA_MAINNET_CAIP2 => Ok("SOL"),
        id if id.starts_with("eip155:") => Ok(id.trim_start_matches("eip155:")),
        _ => Err(RpcError::InvalidValue(caip2.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caip2_to_lifi_chain_id() {
        assert!(matches!(caip2_to_lifi_chain_id("eip155:1"), Ok("1")));
        assert!(matches!(
            caip2_to_lifi_chain_id("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"),
            Ok("SOL")
        ));
    }
}
