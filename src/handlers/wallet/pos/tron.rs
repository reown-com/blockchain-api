use {
    super::{AssetNamespaceType, BuildPosTxError, BuildTransactionParams, BuildTransactionResult,
        TransactionBuilder,
        ValidatedTransactionParams,
        TransactionId,
        TransactionRpc,
        TransactionStatus,
    },
    crate::{state::AppState, utils::crypto::Caip2ChainId},
    alloy::{
        primitives::{utils::parse_units, Address as EthAddress, U256},
        sol,
        sol_types::SolCall,
    },
    async_trait::async_trait,
    axum::extract::State,
    bs58,
    hex,
    serde::{Deserialize},
    std::sync::Arc,
    strum_macros::EnumString,
    tracing::debug,
};

const TRON_BASE_URL: &str = "https://nile.trongrid.io";
const TRON_SIGN_TRANSACTION_METHOD: &str = "tron_signTransaction";
const DEFAULT_FEE_LIMIT: u64 = 200_000_000;

sol! {
    function transfer(address to, uint256 value) external returns (bool);
    function approve(address spender, uint256 value) external returns (bool);
}

#[derive(Debug, Deserialize)]
struct TronRpcBuildTransactionResponse {
    transaction: serde_json::Value,
}


#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AssetNamespace {
    Trx20,
    Slip44,
}

impl AssetNamespaceType for AssetNamespace {
    fn is_native(&self) -> bool {
        matches!(self, AssetNamespace::Slip44)
    }
}

pub struct TronTransactionBuilder;

#[async_trait]
impl TransactionBuilder for TronTransactionBuilder {
    fn namespace(&self) -> &'static str {
        "tron"
    }

    async fn build(
        &self,
        state: State<Arc<AppState>>,
        project_id: String,
        params: BuildTransactionParams,
    ) -> Result<BuildTransactionResult, BuildPosTxError> {
        let validated_params: ValidatedTransactionParams<AssetNamespace> =
            ValidatedTransactionParams::validate_params(&params)?;

        match validated_params.namespace {
            AssetNamespace::Trx20 => {
                build_trx20_transfer(state, validated_params, &params.amount, &project_id).await
            }
            _ => {
                return Err(BuildPosTxError::Validation(
                    "Unsupported asset namespace".to_string(),
                ));
            }
        }
    }
}

async fn build_trx20_transfer(
    State(state): State<Arc<AppState>>,
    params: ValidatedTransactionParams<AssetNamespace>,
    amount: &str,
    _project_id: &str,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    let to_eth = tron_base58_to_eth_address(&params.recipient_address)?;
    let token_amount = parse_token_amount(amount)?;
    
    let data = transferCall {
        to: to_eth,
        value: token_amount,
    }.abi_encode();

    let params_hex = &hex::encode(&data[4..]);

    let owner_address = tron_b58_to_hex41(&params.sender_address)?;
    let contract_address = tron_b58_to_hex41(&params.asset.asset_reference())?;


    let url = format!("{}/wallet/triggersmartcontract", TRON_BASE_URL);
    let body = serde_json::json!({
        "owner_address": owner_address,
        "contract_address": contract_address,
        "function_selector": "transfer(address,uint256)",
        "parameter": params_hex,
        "fee_limit": DEFAULT_FEE_LIMIT,
        "call_value": 0,
        "visible": false
    });
        
    let resp: TronRpcBuildTransactionResponse = state.http_client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to send request: {}", e)))?
        .error_for_status()
        .map_err(|e| BuildPosTxError::Internal(format!("HTTP error: {}", e)))?
        .json()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse response: {}", e)))?;
    
    debug!("tron build transaction resp: {:?}", resp);

    let transaction_rpc = TransactionRpc {
        method: TRON_SIGN_TRANSACTION_METHOD.to_string(),
        params: serde_json::json!({
            "address": params.sender_address,
            "transaction": {
                "result": {
                    "result": true
                },
                "transaction": resp.transaction
            }
        }),
    };
    
    let id = TransactionId::new(params.asset.chain_id()).to_string();
    
    Ok(BuildTransactionResult {
        transaction_rpc,
        id,
    })
}

fn parse_token_amount(amount: &str) -> Result<U256, BuildPosTxError> {
    let parsed_value = parse_units(amount, 6).map_err(|e| {
        BuildPosTxError::Validation(format!(
            "Unable to parse amount with 6 decimals: {}",
            e
        ))
    })?;

    Ok(parsed_value.into())
}

pub async fn get_transaction_status(
    State(state): State<Arc<AppState>>,
    _project_id: &str,
    response: &str,
    _chain_id: &Caip2ChainId,
) -> Result<TransactionStatus, BuildPosTxError> {
    let signed_tx: serde_json::Value = serde_json::from_str(response)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid wallet response: {}", e)))?;

    let txid = signed_tx
        .get("txID")
        .and_then(|v| v.as_str())
        .ok_or_else(|| BuildPosTxError::Validation("Missing txID in wallet response".to_string()))?;

    let get_tx_url = format!("{}/wallet/gettransactionbyid", TRON_BASE_URL);
    let get_tx_body = serde_json::json!({ "value": txid });
    let existing_tx: serde_json::Value = state
        .http_client
        .post(&get_tx_url)
        .json(&get_tx_body)
        .send()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to send request: {}", e)))?
        .error_for_status()
        .map_err(|e| BuildPosTxError::Internal(format!("HTTP error: {}", e)))?
        .json()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse response: {}", e)))?;

    let already_broadcasted = match &existing_tx {
        serde_json::Value::Object(map) => !map.is_empty(),
        _ => false,
    };

    if !already_broadcasted {
        let broadcast_url = format!("{}/wallet/broadcasttransaction", TRON_BASE_URL);
        let broadcast_resp: serde_json::Value = state
            .http_client
            .post(&broadcast_url)
            .json(&signed_tx)
            .send()
            .await
            .map_err(|e| BuildPosTxError::Internal(format!("Failed to send request: {}", e)))?
            .error_for_status()
            .map_err(|e| BuildPosTxError::Internal(format!("HTTP error: {}", e)))?
            .json()
            .await
            .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse response: {}", e)))?;

        debug!("tron broadcast resp: {:?}", broadcast_resp);

        let ok = broadcast_resp.get("result").and_then(|v| v.as_bool()).unwrap_or(false);
        if !ok {
            let code = broadcast_resp
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let message = broadcast_resp
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            return Err(BuildPosTxError::Internal(format!(
                "Broadcast failed: {} {}",
                code, message
            )));
        }

        return Ok(TransactionStatus::Pending);
    }

    let info_url = format!("{}/wallet/gettransactioninfobyid", TRON_BASE_URL);
    let info_body = serde_json::json!({ "value": txid });
    let info_resp: serde_json::Value = state
        .http_client
        .post(&info_url)
        .json(&info_body)
        .send()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to send request: {}", e)))?
        .error_for_status()
        .map_err(|e| BuildPosTxError::Internal(format!("HTTP error: {}", e)))?
        .json()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse response: {}", e)))?;

    debug!("tron tx info resp: {:?}", info_resp);

    match &info_resp {
        serde_json::Value::Object(map) if map.is_empty() => Ok(TransactionStatus::Pending),
        serde_json::Value::Object(_) => {
            if let Some(receipt) = info_resp.get("receipt") {
                if let Some(result) = receipt.get("result").and_then(|v| v.as_str()) {
                    return match result {
                        "SUCCESS" => Ok(TransactionStatus::Confirmed),
                        _ => Ok(TransactionStatus::Failed),
                    };
                }
            }
            if info_resp.get("blockNumber").is_some() {
                return Ok(TransactionStatus::Confirmed);
            }
            Ok(TransactionStatus::Pending)
        }
        _ => Ok(TransactionStatus::Pending),
    }
}

fn tron_base58_to_eth_address(b58: &str) -> Result<EthAddress, BuildPosTxError> {
    let bytes = bs58::decode(b58).with_check(None).into_vec().map_err(|e| BuildPosTxError::Validation(format!("Failed to decode TRON address: {}", e)))?;
    if bytes.len() != 21 || bytes[0] != 0x41 {
        return Err(BuildPosTxError::Validation("invalid TRON address payload".to_string()));
    }
    Ok(EthAddress::from_slice(&bytes[1..21]))
}

fn tron_b58_to_hex41(b58: &str) -> Result<String, BuildPosTxError> {
    let bytes = bs58::decode(b58).with_check(None).into_vec().map_err(|e| BuildPosTxError::Validation(format!("Failed to decode TRON address: {}", e)))?;
    if bytes.len() != 21 || bytes[0] != 0x41 {
        return Err(BuildPosTxError::Validation("invalid TRON address".to_string()));
    }
    Ok(format!("{}", hex::encode(bytes))) 
}