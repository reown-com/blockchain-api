use {
    crate::{
        error::RpcError,
        utils::sessions::{
            extract_addresses_from_execution_batch, extract_values_sum_from_execution_batch,
            ExecutionTransaction,
        },
    },
    alloy::primitives::{Address, U256},
    serde::{Deserialize, Serialize},
    serde_json::Value,
    strum_macros::{Display, EnumIter, EnumString},
    tracing::error,
};

/// Supported permission types
#[derive(Clone, Copy, Debug, EnumString, EnumIter, Display, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum PermissionType {
    ContractCall,
    NativeTokenTransfer,
}

/// `contract-call` permission type data schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractCallPermissionData {
    pub address: Address,
    pub abi: Value,
    pub functions: Value,
}

/// `native-token-transfer` permission type data schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeTokenTransferPermissionData {
    pub allowance: U256,
    pub start: usize,
    pub period: usize,
}

/// `contract-call` permission type check
pub fn contract_call_permission_check(
    execution_batch: Vec<ExecutionTransaction>,
    contract_call_permission_data: ContractCallPermissionData,
) -> Result<(), RpcError> {
    let execution_addresses = extract_addresses_from_execution_batch(execution_batch)?;
    let call_address = contract_call_permission_data.address;

    for address in execution_addresses {
        if address != call_address {
            error!("Execution address does not match the contract address in the permission data. Execution Address: {:?}, Contract Address: {:?}", address, call_address);
            return Err(RpcError::CosignerPermissionDenied(format!(
              "Execution address does not match the contract address in the permission data. Execution Address: {:?}, Contract Address: {:?}", address, call_address
          )));
        }
    }
    Ok(())
}

/// `native-token-transfer` permission type check
pub fn native_token_transfer_permission_check(
    execution_batch: Vec<ExecutionTransaction>,
    native_token_transfer_permission_data: NativeTokenTransferPermissionData,
) -> Result<(), RpcError> {
    let allowance = native_token_transfer_permission_data.allowance;
    let sum: U256 = extract_values_sum_from_execution_batch(execution_batch)?;
    if sum > allowance {
        error!(
            "Execution value is greater than the allowance. Execution Value: {:?}, Allowance: {:?}",
            sum, allowance
        );
        return Err(RpcError::CosignerPermissionDenied(format!(
            "Execution value is greater than the allowance. Execution Value: {:?}, Allowance: {:?}",
            sum, allowance
        )));
    }

    Ok(())
}
