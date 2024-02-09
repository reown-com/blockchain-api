use {
    ethers::types::H160,
    std::{collections::HashMap, str::FromStr},
};

/// Veryfy message signature signed by the keccak256
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

/// Convert EVM chain ID to coin type ENSIP-11
#[tracing::instrument]
pub fn convert_evm_chain_id_to_coin_type(chain_id: u32) -> u32 {
    0x80000000 | chain_id
}

/// Convert coin type ENSIP-11 to EVM chain ID
#[tracing::instrument]
pub fn convert_coin_type_to_evm_chain_id(coin_type: u32) -> u32 {
    0x7FFFFFFF & coin_type
}

/// Convert from human readable chain id (e.g. polygon) to CAIP-2 format
/// (e.g. eip155:137)
pub fn string_chain_id_to_caip2_format(chain_id: &str) -> Result<String, anyhow::Error> {
    // Aliases for string chain ids
    let aliases: HashMap<String, Vec<String>> =
        HashMap::from([("mainnet".into(), vec!["ethereum".into()])]);

    for (alias_name, aliases_vec) in aliases {
        if aliases_vec.contains(&chain_id.to_lowercase()) {
            return Ok(format!(
                "eip155:{}",
                ethers::types::Chain::from_str(&alias_name)? as u64
            ));
        }
    }

    Ok(format!(
        "eip155:{}",
        ethers::types::Chain::from_str(chain_id)? as u64
    ))
}

/// Compare two values (either H160 or &str) in constant time to prevent timing
/// attacks
pub fn constant_time_eq(a: impl AsRef<[u8]>, b: impl AsRef<[u8]>) -> bool {
    let a_bytes = a.as_ref();
    let b_bytes = b.as_ref();

    if a_bytes.len() != b_bytes.len() {
        return false;
    }

    let mut result = 0;
    for (byte_a, byte_b) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        ethers::types::H160,
        std::{collections::HashMap, str::FromStr},
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
    fn test_convert_coin_type_to_evm_chain_id() {
        // Polygon
        let chain_id = 137;
        let coin_type = 2147483785;
        assert_eq!(convert_evm_chain_id_to_coin_type(chain_id), coin_type);
        assert_eq!(convert_coin_type_to_evm_chain_id(coin_type), chain_id);
    }

    #[test]
    fn test_string_chain_id_to_caip2_format() {
        let mut chains: HashMap<&str, u64> = HashMap::new();
        chains.insert("mainnet", 1);
        // Test for an `ethereum` alias
        chains.insert("ethereum", 1);
        chains.insert("goerli", 5);
        chains.insert("optimism", 10);
        chains.insert("bsc", 56);
        chains.insert("xdai", 100);
        chains.insert("polygon", 137);
        chains.insert("base", 8453);

        for (chain_id, coin_type) in chains.iter() {
            let result = string_chain_id_to_caip2_format(chain_id);
            assert!(result.is_ok(), "chain_id is not found: {}", chain_id);
            assert_eq!(result.unwrap(), format!("eip155:{}", coin_type));
        }
    }

    #[test]
    fn test_constant_time_eq() {
        let string_one = "some string";
        let string_two = "some another string";
        assert!(!constant_time_eq(string_one, string_two));
        assert!(constant_time_eq(string_one, string_one));
    }
}
