use {
    super::{
        AssetNamespaceType, BuildPosTxError, BuildTransactionParams, BuildTransactionResult,
        TransactionBuilder, TransactionId, TransactionRpc, TransactionStatus,
        ValidatedTransactionParams,
    },
    crate::{state::AppState, utils::crypto::Caip2ChainId},
    alloy::{
        primitives::{utils::parse_units, Address as EthAddress, U256},
        sol,
        sol_types::SolCall,
    },
    async_trait::async_trait,
    axum::extract::State,
    bs58, hex,
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    std::sync::Arc,
    strum_macros::EnumString,
    tracing::debug,
};

const TRON_SIGN_TRANSACTION_METHOD: &str = "tron_signTransaction";
const DEFAULT_FEE_LIMIT: u64 = 200_000_000;

static TRON_NETWORK_URL: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("0x2b6653dc", "https://api.trongrid.io"),  // Mainnet
        ("0xcd8690dc", "https://nile.trongrid.io"), // Testnet
    ])
});

sol! {
    function transfer(address to, uint256 value) external returns (bool);
    function approve(address spender, uint256 value) external returns (bool);
}

#[derive(Debug, Serialize)]
struct TronTriggerSmartContractRequest {
    owner_address: String,
    contract_address: String,
    function_selector: String,
    parameter: String,
    fee_limit: u64,
    call_value: u64,
    visible: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TronSignedTransaction {
    raw_data: serde_json::Value,
    raw_data_hex: String,
    #[serde(rename = "txID")]
    tx_id: String,
    visible: Option<bool>,
    signature: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct TronTriggerSmartContractResponse {
    transaction: TronSignedTransaction,
}

#[derive(Debug, Serialize)]
struct TronGetByIdRequest {
    value: String,
}

#[derive(Debug, Deserialize)]
struct TronGetTransactionByIdResponse {
    #[serde(rename = "txID")]
    tx_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TronBroadcastTransactionResponse {
    result: bool,
    code: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TronReceipt {
    result: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TronGetTransactionInfoByIdResponse {
    id: Option<String>,
    #[serde(rename = "blockNumber")]
    block_number: Option<u64>,
    receipt: Option<TronReceipt>,
}

fn get_provider_url(chain_id: &Caip2ChainId) -> Result<String, BuildPosTxError> {
    if chain_id.namespace() != "tron" {
        return Err(BuildPosTxError::Validation(
            "Provider not supported".to_string(),
        ));
    }

    match TRON_NETWORK_URL.get(chain_id.reference()) {
        Some(url) => Ok(url.to_string()),
        _ => Err(BuildPosTxError::Validation(
            "Provider not supported".to_string(),
        )),
    }
}

async fn tron_trigger_smart_contract(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    req: &TronTriggerSmartContractRequest,
) -> Result<TronTriggerSmartContractResponse, BuildPosTxError> {
    let base = get_provider_url(chain_id)?;
    let url = format!("{}/wallet/triggersmartcontract", base);
    let resp = state
        .http_client
        .post(&url)
        .json(req)
        .send()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to send request: {}", e)))?
        .error_for_status()
        .map_err(|e| BuildPosTxError::Internal(format!("HTTP error: {}", e)))?
        .json::<TronTriggerSmartContractResponse>()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse response: {}", e)))?;
    Ok(resp)
}

async fn tron_get_transaction_by_id(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    txid: &str,
) -> Result<Option<TronGetTransactionByIdResponse>, BuildPosTxError> {
    let base = get_provider_url(chain_id)?;
    let url = format!("{}/wallet/gettransactionbyid", base);
    let body = TronGetByIdRequest {
        value: txid.to_string(),
    };
    let resp = state
        .http_client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to send request: {}", e)))?
        .error_for_status()
        .map_err(|e| BuildPosTxError::Internal(format!("HTTP error: {}", e)))?
        .json::<TronGetTransactionByIdResponse>()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse response: {}", e)))?;

    if resp.tx_id.is_some() {
        Ok(Some(resp))
    } else {
        Ok(None)
    }
}

async fn tron_broadcast_transaction(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    tx: &TronSignedTransaction,
) -> Result<TronBroadcastTransactionResponse, BuildPosTxError> {
    let base = get_provider_url(chain_id)?;
    let url = format!("{}/wallet/broadcasttransaction", base);
    let resp = state
        .http_client
        .post(&url)
        .json(tx)
        .send()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to send request: {}", e)))?
        .error_for_status()
        .map_err(|e| BuildPosTxError::Internal(format!("HTTP error: {}", e)))?
        .json::<TronBroadcastTransactionResponse>()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse response: {}", e)))?;
    Ok(resp)
}

async fn tron_get_transaction_info_by_id(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    txid: &str,
) -> Result<Option<TronGetTransactionInfoByIdResponse>, BuildPosTxError> {
    let base = get_provider_url(chain_id)?;
    let url = format!("{}/wallet/gettransactioninfobyid", base);
    let body = TronGetByIdRequest {
        value: txid.to_string(),
    };
    let resp = state
        .http_client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to send request: {}", e)))?
        .error_for_status()
        .map_err(|e| BuildPosTxError::Internal(format!("HTTP error: {}", e)))?
        .json::<TronGetTransactionInfoByIdResponse>()
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse response: {}", e)))?;

    if resp.id.is_some() || resp.block_number.is_some() || resp.receipt.is_some() {
        Ok(Some(resp))
    } else {
        Ok(None)
    }
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
    state: State<Arc<AppState>>,
    params: ValidatedTransactionParams<AssetNamespace>,
    amount: &str,
    _project_id: &str,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    let to_eth = tron_base58_to_eth_address(&params.recipient_address)?;
    let token_amount = parse_token_amount(amount)?;

    let data = transferCall {
        to: to_eth,
        value: token_amount,
    }
    .abi_encode();

    let params_hex = &hex::encode(&data[4..]);

    let owner_address = tron_b58_to_hex41(&params.sender_address)?;
    let contract_address = tron_b58_to_hex41(&params.asset.asset_reference())?;

    let trigger_req = TronTriggerSmartContractRequest {
        owner_address,
        contract_address,
        function_selector: "transfer(address,uint256)".to_string(),
        parameter: params_hex.to_string(),
        fee_limit: DEFAULT_FEE_LIMIT,
        call_value: 0,
        visible: false,
    };

    let resp = tron_trigger_smart_contract(state, params.asset.chain_id(), &trigger_req).await?;

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
        BuildPosTxError::Validation(format!("Unable to parse amount with 6 decimals: {}", e))
    })?;

    Ok(parsed_value.into())
}

pub async fn get_transaction_status(
    state: State<Arc<AppState>>,
    _project_id: &str,
    response: &str,
    chain_id: &Caip2ChainId,
) -> Result<TransactionStatus, BuildPosTxError> {
    let signed_tx: TronSignedTransaction = serde_json::from_str(response)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid wallet response: {}", e)))?;

    let txid = signed_tx.tx_id.as_str();

    let already_broadcasted = tron_get_transaction_by_id(state.clone(), chain_id, txid)
        .await?
        .is_some();

    if !already_broadcasted {
        let broadcast_resp = tron_broadcast_transaction(state, chain_id, &signed_tx).await?;
        debug!("tron broadcast resp: {:?}", broadcast_resp);
        if !broadcast_resp.result {
            return Err(BuildPosTxError::Internal(format!(
                "Broadcast failed: {} {}",
                broadcast_resp.code.unwrap_or_default(),
                broadcast_resp.message.unwrap_or_default()
            )));
        }

        return Ok(TransactionStatus::Pending);
    }

    let info_opt = tron_get_transaction_info_by_id(state, chain_id, txid).await?;

    match info_opt {
        Some(info_resp) => {
            if let Some(receipt) = info_resp.receipt {
                if let Some(result) = receipt.result.as_deref() {
                    return Ok(match result {
                        "SUCCESS" => TransactionStatus::Confirmed,
                        _ => TransactionStatus::Failed,
                    });
                }
            }
        }
        _ => return Ok(TransactionStatus::Pending),
    }
    Ok(TransactionStatus::Pending)
}

fn tron_base58_to_eth_address(b58: &str) -> Result<EthAddress, BuildPosTxError> {
    let bytes = bs58::decode(b58).with_check(None).into_vec().map_err(|e| {
        BuildPosTxError::Validation(format!("Failed to decode TRON address: {}", e))
    })?;
    if bytes.len() != 21 || bytes[0] != 0x41 {
        return Err(BuildPosTxError::Validation(
            "invalid TRON address payload".to_string(),
        ));
    }
    Ok(EthAddress::from_slice(&bytes[1..21]))
}

fn tron_b58_to_hex41(b58: &str) -> Result<String, BuildPosTxError> {
    let bytes = bs58::decode(b58).with_check(None).into_vec().map_err(|e| {
        BuildPosTxError::Validation(format!("Failed to decode TRON address: {}", e))
    })?;
    if bytes.len() != 21 || bytes[0] != 0x41 {
        return Err(BuildPosTxError::Validation(
            "invalid TRON address".to_string(),
        ));
    }
    Ok(format!("{}", hex::encode(bytes)))
}
