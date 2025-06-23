use alloy::{
    primitives::{address, Address, Bytes, U256},
    sol_types::SolValue,
};

// Constants for validation - matching OwnableValidator contract
const MIN_ABI_ENCODED_TUPLE_LENGTH: usize = 64;
const MAX_OWNERS_COUNT: usize = 32; // From OwnableValidator.MAX_OWNERS

// OwnableValidator contract address
pub const OWNABLE_VALIDATOR_ADDRESS: Address = address!("2483da3a338895199e5e538530213157e931bf06");

/// Check if the given address is the OwnableValidator contract
pub fn is_ownable_validator_address(address: Address) -> bool {
    address == OWNABLE_VALIDATOR_ADDRESS
}

/// Validation errors for validator format detection
#[derive(Debug, PartialEq, Eq)]
pub enum ValidatorFormatError {
    /// Data is too short to be a valid encoded format
    DataTooShort,
    /// Failed to decode the data as expected format
    InvalidEncoding,
    /// Threshold is zero which is invalid
    ZeroThreshold,
    /// No owners provided
    EmptyOwners,
    /// Too many owners (exceeds MAX_OWNERS)
    TooManyOwners,
    /// Threshold exceeds number of owners
    ThresholdExceedsOwners,
}

/// Check if data is abi.encode(threshold, owners) format - indicates Ownable
/// Validator
///
/// Validates both format and reasonable limits to prevent DoS attacks.
pub fn is_ownable_validator_format(data: &Bytes) -> Result<bool, ValidatorFormatError> {
    if data.len() < MIN_ABI_ENCODED_TUPLE_LENGTH {
        return Ok(false); // Not an error, just not the expected format
    }

    match <(U256, Vec<Address>)>::abi_decode_params(data, true) {
        Ok((threshold, owners)) => {
            // Validate constraints matching OwnableValidator contract
            let threshold_u64 = threshold.to::<u64>();

            if threshold_u64 == 0 {
                return Err(ValidatorFormatError::ZeroThreshold);
            }
            if owners.is_empty() {
                return Err(ValidatorFormatError::EmptyOwners);
            }
            if owners.len() > MAX_OWNERS_COUNT {
                return Err(ValidatorFormatError::TooManyOwners);
            }
            if threshold_u64 > owners.len() as u64 {
                return Err(ValidatorFormatError::ThresholdExceedsOwners);
            }

            Ok(true)
        }
        Err(_) => Ok(false), // Not an error, just not ownable validator format
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

        assert_eq!(is_ownable_validator_format(&data), Ok(true));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_too_short() {
        let data = bytes!("1234567890"); // Too short
        assert_eq!(is_ownable_validator_format(&data), Ok(false));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_wrong_format() {
        let data = bytes!("1234567890123456789012345678901234567890123456789012345678901234567890"); // 64 bytes but wrong format
        assert_eq!(is_ownable_validator_format(&data), Ok(false));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_zero_threshold() {
        let threshold = U256::from(0); // Invalid: zero threshold
        let owners = vec![address!("1111111111111111111111111111111111111111")];
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert_eq!(
            is_ownable_validator_format(&data),
            Err(ValidatorFormatError::ZeroThreshold)
        );
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_threshold_too_large() {
        let threshold = U256::from(2); // Invalid: threshold > owners.len() (only 1 owner)
        let owners = vec![address!("1111111111111111111111111111111111111111")];
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert_eq!(
            is_ownable_validator_format(&data),
            Err(ValidatorFormatError::ThresholdExceedsOwners)
        );
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_threshold_exceeds_owners() {
        let threshold = U256::from(3); // Invalid: threshold > owners.len()
        let owners = vec![
            address!("1111111111111111111111111111111111111111"),
            address!("2222222222222222222222222222222222222222"),
        ];
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert_eq!(
            is_ownable_validator_format(&data),
            Err(ValidatorFormatError::ThresholdExceedsOwners)
        );
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_empty_owners() {
        let threshold = U256::from(1);
        let owners: Vec<Address> = vec![]; // Invalid: empty owners list
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert_eq!(
            is_ownable_validator_format(&data),
            Err(ValidatorFormatError::EmptyOwners)
        );
    }

    #[test]
    fn test_is_ownable_validator_format_valid_max_owners() {
        let threshold = U256::from(32);
        let owners: Vec<Address> = (0..32) // Valid: exactly MAX_OWNERS_COUNT (32)
            .map(|i| Address::from([i as u8; 20]))
            .collect();
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert_eq!(is_ownable_validator_format(&data), Ok(true));
    }

    #[test]
    fn test_is_ownable_validator_format_invalid_too_many_owners() {
        let threshold = U256::from(1);
        let owners: Vec<Address> = (0..33) // Invalid: exceeds MAX_OWNERS_COUNT (32)
            .map(|i| Address::from([i as u8; 20]))
            .collect();
        let data = Bytes::from((threshold, owners).abi_encode_params());
        assert_eq!(
            is_ownable_validator_format(&data),
            Err(ValidatorFormatError::TooManyOwners)
        );
    }
}
