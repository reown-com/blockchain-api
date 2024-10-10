use {
    crate::error::RpcError,
    ethers::{
        abi::{Abi, Token},
        types::{H160, U256},
    },
};

// Extract the execution batch components from the calldata
// from bundler's `execute` function ABI
pub fn extract_execution_batch_components(
    call_data_bytes: Vec<u8>,
) -> Result<Vec<Token>, RpcError> {
    let execute_abi_json = r#"
    [
      {
          "type": "function",
          "name": "execute",
          "inputs": [
            {
                "name": "execMode",
                "type": "bytes32",
                "internalType": "ExecMode"
            },
            {
                "name": "executionCalldata",
                "type": "bytes",
                "internalType": "bytes"
            }
          ],
          "outputs": [],
          "stateMutability": "payable"
      }
    ]
    "#;

    let execute_abi: Abi = serde_json::from_str(execute_abi_json)?;
    let execute_function = execute_abi.function("execute").map_err(|e| {
        RpcError::AbiDecodingError(format!("Failed to parse execute function: {}", e))
    })?;

    // Verify the function selector
    let function_selector = &call_data_bytes[0..4];
    let expected_selector = execute_function.short_signature();

    if function_selector != expected_selector {
        return Err(RpcError::AbiDecodingError(
            "Function selector does not match `execute`".into(),
        ));
    }

    // Decode the calldata
    let decoded_params = execute_function
        .decode_input(&call_data_bytes[4..])
        .map_err(|e| RpcError::AbiDecodingError(format!("Failed to decode calldata: {}", e)))?;

    // Extract executionCalldata
    let execution_calldata = match &decoded_params[1] {
        Token::Bytes(bytes) => bytes,
        _ => {
            return Err(RpcError::AbiDecodingError(
                "executionCalldata is not bytes".into(),
            ))
        }
    };

    // Define the executionCalldata ABI
    let execution_calldata_abi_json = r#"
    [
      {
        "type": "function",
        "name": "decodeExecutionCalldata",
        "inputs": [
          {
            "name": "executionBatch",
            "type": "tuple[]",
            "components": [
              {
                  "name": "target",
                  "type": "address"
              },
              {
                  "name": "value",
                  "type": "uint256"
              },
              {
                  "name": "callData",
                  "type": "bytes"
              }
            ]
          }
        ],
        "outputs": []
      }
    ]
    "#;

    // Parse the executionCalldata ABI
    let execution_calldata_abi: Abi = serde_json::from_str(execution_calldata_abi_json)?;
    let decode_function = execution_calldata_abi
        .function("decodeExecutionCalldata")
        .map_err(|e| {
            RpcError::AbiDecodingError(format!(
                "Failed to parse decodeExecutionCalldata function: {}",
                e
            ))
        })?;

    // Decode executionCalldata
    let tokens = decode_function
        .decode_input(execution_calldata)
        .map_err(|e| {
            RpcError::AbiDecodingError(format!("Failed to decode executionCalldata: {}", e))
        })?;

    // Extract the execution batch
    let execution_batch = match &tokens[0] {
        Token::Array(arr) => arr,
        _ => {
            return Err(RpcError::AbiDecodingError(
                "Expected an array for executionBatch".into(),
            ))
        }
    };

    if execution_batch.is_empty() {
        return Err(RpcError::AbiDecodingError("executionBatch is empty".into()));
    }

    Ok(execution_batch.clone())
}

/// Extract addresses from the bundler's execute calldata execution batch
pub fn extract_addresses_from_execution_batch(
    execution_batch: Vec<Token>,
) -> Result<Vec<H160>, RpcError> {
    let mut targets = Vec::new();
    for tx in execution_batch {
        let tx = match &tx {
            Token::Tuple(tuple) => tuple,
            _ => {
                return Err(RpcError::AbiDecodingError(
                    "Expected a tuple for execution batch transaction".into(),
                ))
            }
        };
        let target = match &tx[0] {
            Token::Address(addr) => *addr,
            _ => {
                return Err(RpcError::AbiDecodingError(
                    "Expected address as a first field for target in the execution batch item"
                        .into(),
                ))
            }
        };
        targets.push(target);
    }

    Ok(targets)
}

/// Exract values from the bundler's execute calldata execution batch
pub fn extract_values_from_execution_batch(
    execution_batch: Vec<Token>,
) -> Result<Vec<U256>, RpcError> {
    let mut values = Vec::new();
    for tx in execution_batch {
        let tx = match &tx {
            Token::Tuple(tuple) => tuple,
            _ => {
                return Err(RpcError::AbiDecodingError(
                    "Expected a tuple for execution batch transaction".into(),
                ))
            }
        };

        let value = match &tx[1] {
            Token::Uint(value) => *value,
            _ => {
                return Err(RpcError::AbiDecodingError(
                    "Expected value as a second field for value in the execution batch item".into(),
                ))
            }
        };

        values.push(value);
    }
    Ok(values)
}
