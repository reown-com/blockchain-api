use {
    crate::error::RpcError,
    alloy::{
        dyn_abi::DynSolValue,
        primitives::{Address, Bytes, U256},
        sol,
        sol_types::{SolCall, SolType},
    },
    yttrium::smart_accounts::safe::Safe7579,
};

type BatchTransactionType = sol! {
    (address, uint256, bytes)[]
};

// Extract the execution batch components from the calldata
// from bundler's `execute` function ABI
pub fn extract_execution_batch_components(
    call_data_bytes: &[u8],
) -> Result<Vec<DynSolValue>, RpcError> {
    // Decode the calldata into Safe7579::executeCall
    let execute_call = Safe7579::executeCall::abi_decode(call_data_bytes, true).map_err(|e| {
        RpcError::AbiDecodingError(format!("Failed to decode executeCall: {:?}", e))
    })?;

    // Access the mode bytes directly
    let mode_bytes = &execute_call.mode;

    // Manually parse the mode_bytes
    let batch_flag = mode_bytes[0];
    let _exec_type = mode_bytes[1];
    let is_batch = batch_flag != 0;

    if is_batch {
        let execution_calldata_bytes = &execute_call.executionCalldata;
        let batch_transactions: Vec<(Address, U256, Bytes)> =
            BatchTransactionType::abi_decode(execution_calldata_bytes, true).map_err(|e| {
                RpcError::AbiDecodingError(format!(
                    "Failed to decode batch transactions ABI type: {:?}",
                    e
                ))
            })?;

        let execution_batch = batch_transactions
            .into_iter()
            .map(|(target, value, call_data)| {
                DynSolValue::Tuple(vec![
                    DynSolValue::Address(target),
                    DynSolValue::Uint(value, 256),
                    DynSolValue::Bytes(call_data.to_vec()),
                ])
            })
            .collect();

        Ok(execution_batch)
    } else {
        // Single transaction: executionCalldata is packed-encoded
        let execution_calldata_bytes = &execute_call.executionCalldata;
        if execution_calldata_bytes.len() < 20 + 32 {
            return Err(RpcError::AbiDecodingError(
                "executionCalldata is too short for a single transaction".into(),
            ));
        }

        // Manually parse the packed-encoded single transaction
        // Address (20 bytes)
        let address = Address::from_slice(&execution_calldata_bytes[0..20]);

        // Uint256 value (32 bytes)
        let value_bytes = &execution_calldata_bytes[20..52];
        let value_bytes_array: [u8; 32] = value_bytes.try_into().map_err(|_| {
            RpcError::AbiDecodingError("Invalid value bytes length in execution calldata".into())
        })?;

        // Specify the const generic parameter <32>
        let value = U256::from_be_bytes::<32>(value_bytes_array);

        // callData (remaining bytes)
        let call_data = execution_calldata_bytes[52..].to_vec();

        let transaction = DynSolValue::Tuple(vec![
            DynSolValue::Address(address),
            DynSolValue::Uint(value, 256),
            DynSolValue::Bytes(call_data),
        ]);

        Ok(vec![transaction])
    }
}

/// Extract addresses from the bundler's execute calldata execution batch
pub fn extract_addresses_from_execution_batch(
    execution_batch: Vec<DynSolValue>,
) -> Result<Vec<Address>, RpcError> {
    let mut targets = Vec::with_capacity(execution_batch.len());

    for tx in execution_batch {
        let values = match tx {
            DynSolValue::Tuple(values) => values,
            _ => {
                return Err(RpcError::AbiDecodingError(
                    "Expected a tuple for execution batch transaction".into(),
                ))
            }
        };

        if values.is_empty() {
            return Err(RpcError::AbiDecodingError(
                "Expected non-empty tuple for execution batch transaction".into(),
            ));
        }

        let target =
            match &values[0] {
                DynSolValue::Address(addr) => *addr,
                _ => return Err(RpcError::AbiDecodingError(
                    "Expected address as the first field for target in the execution batch item"
                        .into(),
                )),
            };

        targets.push(target);
    }

    Ok(targets)
}

/// Exract values from the bundler's execute calldata execution batch
pub fn extract_values_from_execution_batch(
    execution_batch: Vec<DynSolValue>,
) -> Result<Vec<U256>, RpcError> {
    let mut values_vec = Vec::with_capacity(execution_batch.len());

    for tx in execution_batch {
        let values = match tx {
            DynSolValue::Tuple(values) => values,
            _ => {
                return Err(RpcError::AbiDecodingError(
                    "Expected a tuple for execution batch transaction".into(),
                ))
            }
        };

        if values.len() <= 1 {
            return Err(RpcError::AbiDecodingError(
                "Expected at least two fields in the execution batch transaction tuple".into(),
            ));
        }

        let value =
            match &values[1] {
                DynSolValue::Uint(value, _) => *value,
                _ => return Err(RpcError::AbiDecodingError(
                    "Expected uint256 as the second field for value in the execution batch item"
                        .into(),
                )),
            };

        values_vec.push(value);
    }
    Ok(values_vec)
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        alloy::primitives::address,
        yttrium::{smart_accounts::safe::get_call_data, transaction::Transaction},
    };

    #[test]
    // Check for the packed calldata format for a single transaction
    fn single_execution_call_data_value() {
        let encoded_data = get_call_data(vec![Transaction {
            to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            value: U256::from(1010101),
            data: Bytes::new(),
        }]);
        let decoded_data = extract_execution_batch_components(&encoded_data).unwrap();
        assert_eq!(decoded_data.len(), 1);
    }

    #[test]
    // Check for the regular calldata format for multiple transactions
    fn multiple_execution_call_data_value() {
        let encoded_data = get_call_data(vec![
            Transaction {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                value: U256::from(1010101),
                data: Bytes::new(),
            },
            Transaction {
                to: address!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab"),
                value: U256::from(2020202),
                data: Bytes::new(),
            },
        ]);
        let decoded_data = extract_execution_batch_components(&encoded_data).unwrap();
        assert_eq!(decoded_data.len(), 2);
    }
}
