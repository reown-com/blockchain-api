use {
    super::{
        AssetNamespaceType, BuildPosTxsError, CheckPosTxError, CheckTransactionResult,
        ExecutionError, InternalError, PaymentIntent, RpcError, SupportedNamespace,
        TransactionBuilder, TransactionId, TransactionRpc, TransactionStatus,
        ValidatedPaymentIntent, ValidationError,
    },
    crate::{analytics::MessageSource, state::AppState, utils::crypto::Caip2ChainId},
    alloy::{
        primitives::{utils::parse_units, Address as EthAddress, U256},
        sol,
        sol_types::SolCall,
    },
    async_trait::async_trait,
    axum::extract::State,
    bs58, hex,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    strum::{EnumIter, IntoEnumIterator},
    strum_macros::{Display, EnumString},
    tracing::debug,
};

const TRON_SIGN_TRANSACTION_METHOD: &str = "tron_signTransaction";
const BASE_URL: &str = "https://rpc.walletconnect.org/v1";
const DEFAULT_CHECK_IN: usize = 400;
const FEE_MARGIN_BPS: u16 = 2000;
const BPS_DEN: u128 = 10_000;
const NAMESPACE_NAME: &str = "tron";

sol! {
    function transfer(address to, uint256 value) external returns (bool);
    function approve(address spender, uint256 value) external returns (bool);
}

#[derive(Debug, Serialize, Clone)]
struct BuildTransactionParams {
    from: String,
    to: String,
    data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    gas: Option<String>,
    value: String,
    #[serde(rename = "tokenId")]
    token_id: u64,
    #[serde(rename = "tokenValue")]
    token_value: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SignedTransaction {
    raw_data: serde_json::Value,
    raw_data_hex: String,
    #[serde(rename = "txID")]
    tx_id: String,
    visible: Option<bool>,
    signature: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct BuildTransactionResponse {
    transaction: SignedTransaction,
}

#[derive(Debug, Serialize)]
struct EthCallParams {
    from: String,
    to: String,
    data: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcResponse<T> {
    jsonrpc: Option<String>,
    id: Option<serde_json::Value>,
    #[serde(flatten)]
    payload: JsonRpcPayload<T>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonRpcPayload<T> {
    Success { result: T },
    Error { error: JsonRpcError },
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct TransactionReceipt {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BroadcastResult {
    result: Option<bool>,
    message: Option<String>,
    code: Option<String>,
}

fn get_rpc_url(chain_id: &Caip2ChainId, project_id: &str) -> String {
    format!(
        "{BASE_URL}?chainId={chain_id}&projectId={project_id}&source={}",
        MessageSource::WalletBuildPosTx,
    )
}

async fn call_json_rpc<T: for<'de> Deserialize<'de>>(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<T, RpcError> {
    let url = get_rpc_url(chain_id, project_id);

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    let response = state
        .http_client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| RpcError::Internal(format!("Failed to send request: {}", e)))?;

    let status = response.status();
    let response = response.error_for_status().map_err(|e| {
        if status.is_client_error() {
            debug!("TRON JSON RPC {} error: {}", status, e);
            RpcError::InvalidResponse(format!("HTTP {} error: {}", status, e))
        } else {
            debug!("TRON JSON RPC {} error: {}", status, e);
            RpcError::Internal(format!("HTTP {} error: {}", status, e))
        }
    })?;

    let rpc_response: JsonRpcResponse<T> = response.json().await.map_err(|e| {
        debug!(
            "Failed to parse JSON-RPC response: {} method: {} params: {}",
            e, method, params
        );
        RpcError::InvalidResponse(format!("Failed to parse response: {}", e))
    })?;

    match rpc_response.payload {
        JsonRpcPayload::Success { result } => Ok(result),
        JsonRpcPayload::Error { error } => {
            let error_code = error.code;
            match error_code {
                -32600 => Err(RpcError::InvalidResponse(format!(
                    "Invalid Request: {}",
                    error.message
                ))),
                -32602 => Err(RpcError::InvalidResponse(format!(
                    "Invalid params: {}",
                    error.message
                ))),
                _ => {
                    debug!(
                        "TRON JSON RPC Internal RPC error {}: {}, {}, {}",
                        error_code, error.message, method, params
                    );
                    Err(RpcError::Internal(format!(
                        "RPC error {}: {}",
                        error_code, error.message
                    )))
                }
            }
        }
    }
}

async fn build_transaction(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    params: BuildTransactionParams,
) -> Result<BuildTransactionResponse, RpcError> {
    call_json_rpc(
        state,
        chain_id,
        project_id,
        "buildTransaction",
        serde_json::json!([params]),
    )
    .await
}

async fn eth_call(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    params: EthCallParams,
) -> Result<String, RpcError> {
    call_json_rpc(
        state,
        chain_id,
        project_id,
        "eth_call",
        serde_json::json!([params, "latest"]),
    )
    .await
}

async fn get_transaction_by_hash(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    txid: &str,
) -> Result<Option<serde_json::Value>, RpcError> {
    call_json_rpc(
        state,
        chain_id,
        project_id,
        "eth_getTransactionByHash",
        serde_json::json!([txid]),
    )
    .await
}

async fn broadcast_transaction(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    tx: &SignedTransaction,
) -> Result<BroadcastResult, RpcError> {
    let params = serde_json::json!([
        tx.tx_id,
        tx.visible.unwrap_or(false),
        serde_json::to_string(&tx.raw_data).unwrap_or_default(),
        tx.raw_data_hex,
        tx.signature.clone().unwrap_or_default()
    ]);

    call_json_rpc(
        state,
        chain_id,
        project_id,
        "tron_broadcastTransaction",
        params,
    )
    .await
}

async fn get_transaction_receipt(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    txid: &str,
) -> Result<Option<TransactionReceipt>, RpcError> {
    call_json_rpc(
        state,
        chain_id,
        project_id,
        "eth_getTransactionReceipt",
        serde_json::json!([txid]),
    )
    .await
}

async fn estimate_gas(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    params: BuildTransactionParams,
) -> Result<String, RpcError> {
    let from_eth = hex41_to_eth_hex(&params.from);
    let to_eth = hex41_to_eth_hex(&params.to);

    let eth_params = serde_json::json!({
        "from": from_eth,
        "to": to_eth,
        "data": params.data,
        "value": params.value
    });

    call_json_rpc(
        state,
        chain_id,
        project_id,
        "eth_estimateGas",
        serde_json::json!([eth_params]),
    )
    .await
}

async fn get_gas_price(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
) -> Result<String, RpcError> {
    call_json_rpc(
        state,
        chain_id,
        project_id,
        "eth_gasPrice",
        serde_json::json!([]),
    )
    .await
}

async fn estimate_trc20_fee_limit(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    params: &BuildTransactionParams,
) -> Result<String, BuildPosTxsError> {
    let gas_estimate = estimate_gas(state, chain_id, project_id, params.clone())
        .await
        .map_err(BuildPosTxsError::Rpc)?;

    let gas_price = get_gas_price(state, chain_id, project_id)
        .await
        .map_err(BuildPosTxsError::Rpc)?;

    let gas_value =
        u128::from_str_radix(gas_estimate.trim_start_matches("0x"), 16).map_err(|e| {
            BuildPosTxsError::Execution(ExecutionError::GasEstimation(format!(
                "Failed to parse gas estimate: {}",
                e
            )))
        })?;

    let price_value =
        u128::from_str_radix(gas_price.trim_start_matches("0x"), 16).map_err(|e| {
            BuildPosTxsError::Execution(ExecutionError::GasEstimation(format!(
                "Failed to parse gas price: {}",
                e
            )))
        })?;

    let fee_limit =
        compute_fee_limit(gas_value, price_value).map_err(BuildPosTxsError::Internal)?;

    Ok(format!("0x{:x}", fee_limit))
}

fn compute_fee_limit(gas_estimate: u128, gas_price: u128) -> Result<u128, InternalError> {
    let total = gas_estimate
        .checked_mul(gas_price)
        .and_then(|base| base.checked_mul(BPS_DEN + FEE_MARGIN_BPS as u128))
        .and_then(|v| v.checked_div(BPS_DEN))
        .ok_or_else(|| InternalError::Internal("fee_limit overflow".to_string()))?;

    Ok(total)
}

#[derive(Debug, Clone, PartialEq, EnumString, Display, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum AssetNamespace {
    Trc20,
    Slip44,
}

impl AssetNamespaceType for AssetNamespace {
    fn is_native(&self) -> bool {
        matches!(self, AssetNamespace::Slip44)
    }
}

pub struct TronTransactionBuilder;

#[async_trait]
impl TransactionBuilder<AssetNamespace> for TronTransactionBuilder {
    fn namespace(&self) -> &'static str {
        "tron"
    }
    async fn validate_and_build(
        &self,
        _state: State<Arc<AppState>>,
        project_id: String,
        params: PaymentIntent,
    ) -> Result<TransactionRpc, BuildPosTxsError> {
        let validated_params = ValidatedPaymentIntent::validate_params(&params)?;
        self.build(_state, project_id, validated_params).await
    }

    async fn build(
        &self,
        state: State<Arc<AppState>>,
        project_id: String,
        params: ValidatedPaymentIntent<AssetNamespace>,
    ) -> Result<TransactionRpc, BuildPosTxsError> {
        match params.namespace {
            AssetNamespace::Trc20 => build_trc20_transfer(state, params, &project_id).await,
            _ => {
                return Err(BuildPosTxsError::Validation(ValidationError::InvalidAsset(
                    "Unsupported asset namespace".to_string(),
                )));
            }
        }
    }
}

async fn build_trc20_transfer(
    state: State<Arc<AppState>>,
    params: ValidatedPaymentIntent<AssetNamespace>,
    project_id: &str,
) -> Result<TransactionRpc, BuildPosTxsError> {
    let to_eth = tron_base58_to_eth_address(&params.recipient_address).map_err(|e| {
        BuildPosTxsError::Validation(ValidationError::InvalidRecipient(e.to_string()))
    })?;
    let decimals = fetch_trc20_decimals(
        &state,
        params.asset.chain_id(),
        project_id,
        &params.sender_address,
        params.asset.asset_reference(),
    )
    .await?;
    let token_amount = parse_token_amount(&params.amount, decimals)?;

    debug!(
        "tron build transaction token amount: {:?}, decimals: {:?}",
        token_amount, decimals
    );

    let data = transferCall {
        to: to_eth,
        value: token_amount,
    }
    .abi_encode();

    let data_hex = format!("0x{}", hex::encode(&data));

    let from_address = tron_b58_to_hex41(&params.sender_address)
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidSender(e.to_string())))?;
    let to_address = tron_b58_to_hex41(params.asset.asset_reference())
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string())))?;

    let build_params = BuildTransactionParams {
        from: from_address.clone(),
        to: to_address.clone(),
        data: data_hex.clone(),
        gas: None,
        value: "0x0".to_string(),
        token_id: 0,
        token_value: 0,
    };

    let fee_limit =
        estimate_trc20_fee_limit(&state, params.asset.chain_id(), project_id, &build_params)
            .await?;

    let build_params_with_gas = BuildTransactionParams {
        gas: Some(fee_limit),
        ..build_params
    };

    let resp = build_transaction(
        &state,
        params.asset.chain_id(),
        project_id,
        build_params_with_gas,
    )
    .await
    .map_err(BuildPosTxsError::Rpc)?;

    debug!("tron build transaction resp: {:?}", resp);

    Ok(TransactionRpc {
        id: TransactionId::new(params.asset.chain_id()).to_string(),
        chain_id: params.asset.chain_id().to_string(),
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
    })
}

fn parse_token_amount(amount: &str, decimals: u8) -> Result<U256, BuildPosTxsError> {
    let parsed_value = parse_units(amount, decimals).map_err(|e| {
        BuildPosTxsError::Validation(ValidationError::InvalidAmount(format!(
            "Unable to parse amount with {} decimals: {}",
            decimals, e
        )))
    })?;
    Ok(parsed_value.into())
}

async fn fetch_trc20_decimals(
    state: &State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    project_id: &str,
    owner_b58: &str,
    contract_b58: &str,
) -> Result<u8, BuildPosTxsError> {
    let from_address = tron_b58_to_eth_hex(owner_b58)
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string())))?;
    let to_address = tron_b58_to_eth_hex(contract_b58)
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string())))?;

    let decimals_selector = "0x313ce567";

    let call_params = EthCallParams {
        from: from_address,
        to: to_address,
        data: decimals_selector.to_string(),
    };

    let result = eth_call(state, chain_id, project_id, call_params)
        .await
        .map_err(BuildPosTxsError::Rpc)?;

    let hex_str = result.trim_start_matches("0x");
    let bytes = hex::decode(hex_str).map_err(|e| {
        BuildPosTxsError::Internal(InternalError::Internal(format!(
            "Failed to decode decimals result: {}",
            e
        )))
    })?;

    if bytes.len() < 32 {
        return Err(BuildPosTxsError::Internal(InternalError::Internal(
            format!(
                "Invalid decimals result length: expected 32 bytes, got {}",
                bytes.len()
            ),
        )));
    }

    let decimals = bytes[31];
    Ok(decimals)
}

pub async fn get_transaction_status(
    state: State<Arc<AppState>>,
    project_id: &str,
    signed_tx: &SignedTransaction,
    chain_id: &Caip2ChainId,
) -> Result<TransactionStatus, CheckPosTxError> {
    let txid = signed_tx.tx_id.as_str();

    let already_broadcasted = get_transaction_by_hash(&state, chain_id, project_id, txid)
        .await
        .map_err(CheckPosTxError::Rpc)?
        .is_some();

    if !already_broadcasted {
        let broadcast_resp = broadcast_transaction(&state, chain_id, project_id, signed_tx)
            .await
            .map_err(CheckPosTxError::Rpc)?;
        debug!("tron broadcast resp: {:?}", broadcast_resp);
        if broadcast_resp.result.is_none() || broadcast_resp.result == Some(false) {
            return Err(CheckPosTxError::Rpc(RpcError::Internal(format!(
                "Broadcast failed: {} {}",
                broadcast_resp.code.unwrap_or_default(),
                broadcast_resp.message.unwrap_or_default()
            ))));
        }

        return Ok(TransactionStatus::Pending);
    }

    let receipt_opt = get_transaction_receipt(&state, chain_id, project_id, txid)
        .await
        .map_err(CheckPosTxError::Rpc)?;

    match receipt_opt {
        Some(receipt) => {
            if let Some(status) = receipt.status {
                let status_value =
                    u64::from_str_radix(status.trim_start_matches("0x"), 16).unwrap_or(0);
                return Ok(if status_value == 1 {
                    TransactionStatus::Confirmed
                } else {
                    TransactionStatus::Failed
                });
            }
        }
        _ => return Ok(TransactionStatus::Pending),
    }
    Ok(TransactionStatus::Pending)
}

fn tron_base58_to_eth_address(b58: &str) -> Result<EthAddress, ValidationError> {
    let bytes = bs58::decode(b58).with_check(None).into_vec().map_err(|e| {
        ValidationError::InvalidAddress(format!("Failed to decode TRON address: {}", e))
    })?;
    if bytes.len() != 21 || bytes[0] != 0x41 {
        return Err(ValidationError::InvalidAddress(
            "invalid TRON address payload".to_string(),
        ));
    }
    Ok(EthAddress::from_slice(&bytes[1..21]))
}

fn tron_b58_to_hex41(b58: &str) -> Result<String, ValidationError> {
    let bytes = bs58::decode(b58).with_check(None).into_vec().map_err(|e| {
        ValidationError::InvalidAddress(format!("Failed to decode TRON address: {}", e))
    })?;
    if bytes.len() != 21 || bytes[0] != 0x41 {
        return Err(ValidationError::InvalidAddress(
            "invalid TRON address".to_string(),
        ));
    }
    Ok(hex::encode(bytes).to_string())
}

fn tron_b58_to_eth_hex(b58: &str) -> Result<String, ValidationError> {
    let bytes = bs58::decode(b58).with_check(None).into_vec().map_err(|e| {
        ValidationError::InvalidAddress(format!("Failed to decode TRON address: {}", e))
    })?;
    if bytes.len() != 21 || bytes[0] != 0x41 {
        return Err(ValidationError::InvalidAddress(
            "invalid TRON address".to_string(),
        ));
    }
    Ok(format!("0x{}", hex::encode(&bytes)))
}

fn hex41_to_eth_hex(hex41: &str) -> String {
    if hex41.starts_with("0x") {
        hex41.to_string()
    } else {
        format!("0x{}", hex41)
    }
}

pub async fn check_transaction(
    state: State<Arc<AppState>>,
    project_id: &str,
    response: &str,
    chain_id: &Caip2ChainId,
) -> Result<CheckTransactionResult, CheckPosTxError> {
    let signed_tx: SignedTransaction = serde_json::from_str(response).map_err(|e| {
        CheckPosTxError::Validation(ValidationError::InvalidWalletResponse(format!(
            "Invalid wallet response: {}",
            e
        )))
    })?;

    let status = get_transaction_status(state.clone(), project_id, &signed_tx, chain_id).await?;

    match status {
        TransactionStatus::Pending => Ok(CheckTransactionResult {
            status,
            check_in: Some(DEFAULT_CHECK_IN),
            txid: Some(signed_tx.tx_id),
        }),
        TransactionStatus::Confirmed => Ok(CheckTransactionResult {
            status,
            check_in: None,
            txid: Some(signed_tx.tx_id),
        }),
        TransactionStatus::Failed => Ok(CheckTransactionResult {
            status,
            check_in: None,
            txid: None,
        }),
    }
}

pub fn get_namespace_info() -> SupportedNamespace {
    SupportedNamespace {
        name: NAMESPACE_NAME.to_string(),
        methods: vec![TRON_SIGN_TRANSACTION_METHOD.to_string()],
        events: vec![],
        capabilities: None,
        asset_namespaces: AssetNamespace::iter()
            .map(|x| x.to_string().to_ascii_lowercase())
            .collect(),
    }
}
