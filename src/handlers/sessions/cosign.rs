use {
    super::{super::HANDLER_TASK_METRICS, CoSignRequest, StoragePermissionsItem},
    crate::{
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::crypto::{
            abi_encode_two_bytes_arrays, call_get_user_op_hash, disassemble_caip10,
            is_address_valid, pack_signature, to_eip191_message, CaipNamespaces, ChainId,
            UserOperation,
        },
    },
    axum::{
        extract::{Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::{
        abi::{Abi, Token},
        core::k256::ecdsa::SigningKey,
        signers::LocalWallet,
        types::{Bytes, H160, H256},
        utils::keccak256,
    },
    serde::{Deserialize, Serialize},
    serde_json::{json, Value},
    std::{sync::Arc, time::SystemTime},
    tracing::error,
    wc::future::FutureExt,
};

const ENTRY_POINT_V07_CONTRACT_ADDRESS: &str = "0x0000000071727De22E5E9d8BAf0edAc6f37da032";

/// Co-sign response schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoSignResponse {
    user_operation_tx_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendUserOpRequest {
    pub chain_id: usize,
    pub user_op: UserOperation,
    pub permissions_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendUserOpResponse {
    pub receipt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoSignQueryParams {
    pub project_id: String,
    /// CoSigner version for testing purposes
    pub version: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractCallPermissionData {
    pub address: H160,
    pub abi: Value,
    pub functions: Value,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    address: Path<String>,
    query_payload: Query<CoSignQueryParams>,
    Json(request_payload): Json<CoSignRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, address, request_payload, query_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("sessions_co_sign"))
        .await
}

fn extract_target_from_calldata_using_abi(call_data_bytes: Vec<u8>) -> Result<H160, RpcError> {
    // 1. Define the execute function ABI
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

    // 2. Parse the execute ABI
    let execute_abi: Abi = serde_json::from_str(execute_abi_json)?;
    let execute_function = execute_abi.function("execute").map_err(|e| {
        RpcError::AbiDecodingError(format!("Failed to parse execute function: {}", e))
    })?;

    // 4. Verify the function selector
    let function_selector = &call_data_bytes[0..4];
    let expected_selector = execute_function.short_signature();

    if function_selector != expected_selector {
        return Err(RpcError::AbiDecodingError(
            "Function selector does not match `execute`".into(),
        ));
    }

    // 5. Decode the calldata
    let decoded_params = execute_function
        .decode_input(&call_data_bytes[4..])
        .map_err(|e| RpcError::AbiDecodingError(format!("Failed to decode calldata: {}", e)))?;

    // 6. Extract executionCalldata
    let execution_calldata = match &decoded_params[1] {
        Token::Bytes(bytes) => bytes,
        _ => {
            return Err(RpcError::AbiDecodingError(
                "executionCalldata is not bytes".into(),
            ))
        }
    };

    // 7. Define the executionCalldata ABI
    // Since ethers-rs requires function definitions to parse parameters,
    // we wrap the executionCalldata parameter into a dummy function.
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

    // 8. Parse the executionCalldata ABI
    let execution_calldata_abi: Abi = serde_json::from_str(execution_calldata_abi_json)?;
    let decode_function = execution_calldata_abi
        .function("decodeExecutionCalldata")
        .map_err(|e| {
            RpcError::AbiDecodingError(format!(
                "Failed to parse decodeExecutionCalldata function: {}",
                e
            ))
        })?;

    // 9. Decode executionCalldata
    let tokens = decode_function
        .decode_input(execution_calldata)
        .map_err(|e| {
            RpcError::AbiDecodingError(format!("Failed to decode executionCalldata: {}", e))
        })?;

    // 10. Extract the target address
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

    let first_tx = match &execution_batch[0] {
        Token::Tuple(tuple) => tuple,
        _ => {
            return Err(RpcError::AbiDecodingError(
                "Expected a tuple for transaction".into(),
            ))
        }
    };

    let target = match &first_tx[0] {
        Token::Address(addr) => *addr,
        _ => {
            return Err(RpcError::AbiDecodingError(
                "Expected address for target".into(),
            ))
        }
    };

    Ok(target)
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Path(caip10_address): Path<String>,
    request_payload: CoSignRequest,
    query_payload: Query<CoSignQueryParams>,
) -> Result<Response, RpcError> {
    let project_id = query_payload.project_id.clone();
    state.validate_project_access_and_quota(&project_id).await?;

    // Checking the CAIP-10 address format
    let (namespace, chain_id, address) = disassemble_caip10(&caip10_address)?;
    if namespace != CaipNamespaces::Eip155 {
        return Err(RpcError::UnsupportedNamespace(namespace));
    }
    if !is_address_valid(&address, &namespace) {
        return Err(RpcError::InvalidAddress);
    }

    // ChainID validation
    let chain_id_uint = chain_id
        .parse::<u64>()
        .map_err(|_| RpcError::InvalidChainIdFormat(chain_id.clone()))?;
    if !ChainId::is_supported(chain_id_uint) {
        return Err(RpcError::UnsupportedChain(chain_id.clone()));
    }

    // json stringify request_payload
    error!(
        "request_payload: {:?}",
        serde_json::to_string(&request_payload)
    );

    let chain_id_caip2 = format!("{}:{}", namespace, chain_id);
    let mut user_op = request_payload.user_op.clone();

    // Project ID for internal json-rpc calls
    let rpc_project_id = state
        .config
        .server
        .testing_project_id
        .as_ref()
        .ok_or_else(|| {
            RpcError::InvalidConfiguration(
                "Missing testing project id in the configuration for the cosigner RPC calls"
                    .to_string(),
            )
        })?;

    // Get the userOp hash
    let contract_address = ENTRY_POINT_V07_CONTRACT_ADDRESS
        .parse::<H160>()
        .map_err(|_| RpcError::InvalidAddress)?;
    let user_op_hash = call_get_user_op_hash(
        rpc_project_id,
        &chain_id_caip2,
        contract_address,
        user_op.clone(),
    )
    .await?;
    let eip191_user_op_hash = to_eip191_message(&user_op_hash);

    // Get the PCI object from the IRN
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;
    let irn_call_start = SystemTime::now();
    let storage_permissions_item = irn_client
        .hget(caip10_address.clone(), request_payload.pci.clone())
        .await?
        .ok_or_else(|| {
            RpcError::PermissionNotFound(caip10_address.clone(), request_payload.pci.clone())
        })?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Hget);
    let storage_permissions_item =
        serde_json::from_str::<StoragePermissionsItem>(&storage_permissions_item)?;

    error!(
        "storage_permissions_item: {:?}",
        serde_json::to_string(&storage_permissions_item)
    );

    // Check the permission type
    for permission in storage_permissions_item.permissions {
        if permission.r#type == "contract-call" {
            let call_data = request_payload.user_op.call_data.clone();
            let contract_address = extract_target_from_calldata_using_abi(call_data.to_vec())?;

            println!("Extracted Contract Address: {:?}", contract_address);
            println!(
                "Permission Data: {:?}",
                serde_json::to_string(&permission.data)
            );

            let contract_call_permission_data =
                serde_json::from_value::<ContractCallPermissionData>(permission.data)?;
            if contract_call_permission_data.address != contract_address {
                error!("Contract address does not match the target address in the permission data. UserOp Address: {:?}, Permission Target: {:?}", contract_address, contract_call_permission_data.address);
                return Err(RpcError::CosignerPermissionsDenied(format!(
                    "Contract address does not match the target address in the permission data. UserOp Address: {:?}, Permission Target: {:?}",
                    contract_address, contract_call_permission_data.address.to_string()
                )));
            }
        }
    }

    // Check and get the permission context if it's updated
    let _permission_context = storage_permissions_item
        .context
        .clone()
        .ok_or_else(|| RpcError::PermissionContextNotUpdated(request_payload.pci.clone()))?;

    // Sign the userOp hash with the permission signing key
    let signing_key_bytes = hex::decode(storage_permissions_item.signing_key)
        .map_err(|e| RpcError::WrongHexFormat(e.to_string()))?;
    let signer = SigningKey::from_bytes(signing_key_bytes.as_slice().into())
        .map_err(|e| RpcError::KeyFormatError(e.to_string()))?;

    // Create a LocalWallet for signing and signing the hashed message
    let wallet = LocalWallet::from(signer);
    let signature = wallet
        .sign_hash(H256::from(&keccak256(eip191_user_op_hash.clone())))
        .unwrap();
    let packed_signature = pack_signature(&signature);

    // ABI encode the signatures
    let concatenated_signature = abi_encode_two_bytes_arrays(&packed_signature, &user_op.signature);

    // Update the userOp with the signature
    user_op.signature = concatenated_signature;

    Ok(Json(json!({
        "signature": format!("0x{}", hex::encode(user_op.signature)),
    }))
    .into_response())
}
