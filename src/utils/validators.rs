use alloy::primitives::{address, Address};

// OwnableValidator contract address
pub const OWNABLE_VALIDATOR_ADDRESS: Address = address!("2483da3a338895199e5e538530213157e931bf06");

/// Check if the given address is the OwnableValidator contract
pub fn is_ownable_validator_address(address: Address) -> bool {
    address == OWNABLE_VALIDATOR_ADDRESS
}
