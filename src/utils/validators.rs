use alloy::{
    primitives::{Address, Bytes, U256},
    sol_types::SolValue,
};

// Constants for validation - matching OwnableValidator contract
const MIN_ABI_ENCODED_TUPLE_LENGTH: usize = 64;
const MAX_OWNERS_COUNT: usize = 32; // From OwnableValidator.MAX_OWNERS

/// Check if data is abi.encode(threshold, owners) format - indicates Ownable
/// Validator
///
/// Validates both format and reasonable limits to prevent DoS attacks.
pub fn is_ownable_validator_format(data: &Bytes) -> bool {
    if data.len() < MIN_ABI_ENCODED_TUPLE_LENGTH {
        return false;
    }

    match <(U256, Vec<Address>)>::abi_decode_params(data, true) {
        Ok((threshold, owners)) => {
            // Validate constraints matching OwnableValidator contract
            let threshold_u64 = threshold.to::<u64>();
            threshold_u64 > 0 // Contract: _threshold == 0 check
                && !owners.is_empty()
                && owners.len() <= MAX_OWNERS_COUNT // Contract: ownersLength > MAX_OWNERS check
                && threshold_u64 <= owners.len() as u64 // Contract: ownersLength < _threshold check
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        alloy::primitives::{address, bytes},
    };

    #[test]
    fn test_is_ownable_validator_format_valid() {
        // Create valid OwnableValidator format: abi.encode(threshold, owners)
        let threshold = U256::from(1);
        let owners = vec![
            address!("1111111111111111111111111111111111111111"),
            address!("2222222222222222222222222222222222222222"),
        ];
        let encoded = (threshold, owners).abi_encode_params();
        let data = Bytes::from(encoded);

        assert!(is_ownable_validator_format(&data));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_too_short() {
        let data = bytes!("1234567890"); // Too short
        assert!(!is_ownable_validator_format(&data));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_wrong_format() {
        let data = bytes!("1234567890123456789012345678901234567890123456789012345678901234567890"); // 64 bytes but wrong format
        assert!(!is_ownable_validator_format(&data));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_zero_threshold() {
        let threshold = U256::from(0); // Invalid: zero threshold
        let owners = vec![address!("1111111111111111111111111111111111111111")];
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert!(!is_ownable_validator_format(&data));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_threshold_too_large() {
        let threshold = U256::from(2); // Invalid: threshold > owners.len() (only 1 owner)
        let owners = vec![address!("1111111111111111111111111111111111111111")];
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert!(!is_ownable_validator_format(&data));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_threshold_exceeds_owners() {
        let threshold = U256::from(3); // Invalid: threshold > owners.len()
        let owners = vec![
            address!("1111111111111111111111111111111111111111"),
            address!("2222222222222222222222222222222222222222"),
        ];
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert!(!is_ownable_validator_format(&data));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_empty_owners() {
        let threshold = U256::from(1);
        let owners: Vec<Address> = vec![]; // Invalid: empty owners list
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert!(!is_ownable_validator_format(&data));
    }

    #[test]
    fn test_is_ownable_validator_format_valid_max_owners() {
        let threshold = U256::from(32);
        let owners: Vec<Address> = (0..32) // Valid: exactly MAX_OWNERS_COUNT (32)
            .map(|i| Address::from([i as u8; 20]))
            .collect();
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert!(is_ownable_validator_format(&data));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_too_many_owners() {
        let threshold = U256::from(1);
        let owners: Vec<Address> = (0..33) // Invalid: exceeds MAX_OWNERS_COUNT (32)
            .map(|i| Address::from([i as u8; 20]))
            .collect();
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert!(!is_ownable_validator_format(&data));
    }
}
