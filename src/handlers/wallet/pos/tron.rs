use {
    super::{
        AssetNamespaceType, BuildPosTxsError, CheckPosTxError, CheckTransactionResult,
        ExecutionError, InternalError, PaymentIntent, SupportedNamespace, TransactionBuilder,
        TransactionId, TransactionRpc, TransactionStatus, ValidatedPaymentIntent, ValidationError,
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
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::collections::HashMap,
    std::sync::Arc,
    strum::{EnumIter, IntoEnumIterator},
    strum_macros::{Display, EnumString},
    tracing::debug,
};

const TRON_SIGN_TRANSACTION_METHOD: &str = "tron_signTransaction";
const DEFAULT_CHECK_IN: usize = 400;
const FEE_MARGIN_BPS: u16 = 2000;
const BPS_DEN: u128 = 10_000;
const GET_ENERGY_FEE: &str = "getEnergyFee";
const NAMESPACE_NAME: &str = "tron";

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

#[derive(Debug, Serialize, Clone)]
struct TriggerSmartContractRequest {
    owner_address: String,
    contract_address: String,
    function_selector: String,
    parameter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fee_limit: Option<u64>,
    call_value: u64,
    visible: bool,
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
struct TriggerSmartContractResponse {
    transaction: SignedTransaction,
}

#[derive(Debug, Serialize)]
struct TriggerConstantContractRequest {
    owner_address: String,
    contract_address: String,
    function_selector: String,
    parameter: String,
    visible: bool,
}

#[derive(Debug, Deserialize)]
struct TriggerConstantContractResponse {
    constant_result: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct EstimateEnergyResult {
    result: bool,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EstimateEnergyResponse {
    result: EstimateEnergyResult,
    energy_required: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ChainParameter {
    key: String,
    #[serde(default)]
    value: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ChainParametersResponse {
    #[serde(rename = "chainParameter")]
    chain_parameter: Vec<ChainParameter>,
}

#[derive(Debug, Serialize)]
struct GetByIdRequest {
    value: String,
}

#[derive(Debug, Deserialize)]
struct GetTransactionByIdResponse {
    #[serde(rename = "txID")]
    tx_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BroadcastTransactionResponse {
    result: Option<bool>,
    message: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Receipt {
    result: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GetTransactionInfoByIdResponse {
    id: Option<String>,
    #[serde(rename = "blockNumber")]
    block_number: Option<u64>,
    receipt: Option<Receipt>,
}

fn get_provider_url(chain_id: &Caip2ChainId) -> Result<String, InternalError> {
    if chain_id.namespace() != "tron" {
        return Err(InternalError::InvalidProviderUrl(
            "Provider not supported".to_string(),
        ));
    }

    match TRON_NETWORK_URL.get(chain_id.reference()) {
        Some(url) => Ok(url.to_string()),
        _ => Err(InternalError::InvalidProviderUrl(
            "Provider not supported".to_string(),
        )),
    }
}

async fn post_tron_request<TReq: Serialize, TResp: DeserializeOwned + 'static>(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    path: &str,
    body: &TReq,
) -> Result<TResp, InternalError> {
    let base = get_provider_url(chain_id)?;
    let url = format!("{}{}", base, path);
    let resp = state
        .http_client
        .post(&url)
        .json(body)
        .send()
        .await
        .map_err(|e| InternalError::RpcError(format!("Failed to send request on {}: {}", path, e)))?
        .error_for_status()
        .map_err(|e| InternalError::RpcError(format!("HTTP error on {}: {}", path, e)))?
        .json::<TResp>()
        .await
        .map_err(|e| {
            InternalError::RpcError(format!("Failed to parse response from {}: {}", path, e))
        })?;
    Ok(resp)
}

async fn get_tron_request<TResp: DeserializeOwned + 'static>(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    path: &str,
) -> Result<TResp, InternalError> {
    let base = get_provider_url(chain_id)?;
    let url = format!("{}{}", base, path);
    let resp = state
        .http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| InternalError::RpcError(format!("Failed to send request on {}: {}", path, e)))?
        .error_for_status()
        .map_err(|e| InternalError::RpcError(format!("HTTP error on {}: {}", path, e)))?
        .json::<TResp>()
        .await
        .map_err(|e| {
            InternalError::RpcError(format!("Failed to parse response from {}: {}", path, e))
        })?;
    Ok(resp)
}

async fn trigger_smart_contract(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    req: &TriggerSmartContractRequest,
) -> Result<TriggerSmartContractResponse, InternalError> {
    post_tron_request(state, chain_id, "/wallet/triggersmartcontract", req).await
}

async fn trigger_constant_contract(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    req: &TriggerConstantContractRequest,
) -> Result<TriggerConstantContractResponse, InternalError> {
    post_tron_request(state, chain_id, "/wallet/triggerconstantcontract", req).await
}

async fn get_transaction_by_id(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    txid: &str,
) -> Result<Option<GetTransactionByIdResponse>, InternalError> {
    let body = GetByIdRequest {
        value: txid.to_string(),
    };
    let resp: GetTransactionByIdResponse =
        post_tron_request(state, chain_id, "/wallet/gettransactionbyid", &body).await?;

    if resp.tx_id.is_some() {
        Ok(Some(resp))
    } else {
        Ok(None)
    }
}

async fn broadcast_transaction(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    tx: &SignedTransaction,
) -> Result<BroadcastTransactionResponse, InternalError> {
    post_tron_request(state, chain_id, "/wallet/broadcasttransaction", tx).await
}

async fn get_transaction_info_by_id(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    txid: &str,
) -> Result<Option<GetTransactionInfoByIdResponse>, InternalError> {
    let body = GetByIdRequest {
        value: txid.to_string(),
    };
    let resp: GetTransactionInfoByIdResponse =
        post_tron_request(state, chain_id, "/wallet/gettransactioninfobyid", &body).await?;

    if resp.id.is_some() || resp.block_number.is_some() || resp.receipt.is_some() {
        Ok(Some(resp))
    } else {
        Ok(None)
    }
}

async fn estimate_energy(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    req: &TriggerSmartContractRequest,
) -> Result<EstimateEnergyResponse, InternalError> {
    post_tron_request(state, chain_id, "/wallet/estimateenergy", req).await
}

async fn get_chain_parameters(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
) -> Result<ChainParametersResponse, InternalError> {
    get_tron_request(state, chain_id, "/wallet/getchainparameters").await
}

async fn estimate_trc20_fee_limit(
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    call: &TriggerSmartContractRequest,
) -> Result<u64, BuildPosTxsError> {
    let est = estimate_energy(state.clone(), chain_id, call)
        .await
        .map_err(BuildPosTxsError::Internal)?;
    if !est.result.result {
        let msg = est.result.message.unwrap_or_default();
        return Err(BuildPosTxsError::Internal(InternalError::Internal(
            format!("Energy estimate failed: {}", msg),
        )));
    }
    let energy_required = est.energy_required.ok_or_else(|| {
        BuildPosTxsError::Execution(ExecutionError::GasEstimation(
            "Missing energy_required".to_string(),
        ))
    })?;

    let params = get_chain_parameters(state, chain_id)
        .await
        .map_err(BuildPosTxsError::Internal)?;
    let energy_fee = params
        .chain_parameter
        .into_iter()
        .find(|p| p.key == GET_ENERGY_FEE)
        .and_then(|p| p.value)
        .ok_or_else(|| {
            BuildPosTxsError::Internal(InternalError::Internal(
                "Missing getEnergyFee chain parameter".to_string(),
            ))
        })?;

    let energy_required_u128 = u128::try_from(energy_required).map_err(|_| {
        BuildPosTxsError::Execution(ExecutionError::GasEstimation(
            "negative energy_required".to_string(),
        ))
    })?;
    let energy_fee_u128 = u128::try_from(energy_fee).map_err(|_| {
        BuildPosTxsError::Execution(ExecutionError::GasEstimation(
            "negative getEnergyFee".to_string(),
        ))
    })?;

    compute_fee_limit(energy_required_u128, energy_fee_u128).map_err(BuildPosTxsError::Internal)
}

fn compute_fee_limit(energy_required: u128, energy_fee: u128) -> Result<u64, InternalError> {
    let total = energy_required
        .checked_mul(energy_fee)
        .and_then(|base| base.checked_mul(BPS_DEN + FEE_MARGIN_BPS as u128))
        .and_then(|v| v.checked_div(BPS_DEN))
        .ok_or_else(|| InternalError::Internal("fee_limit overflow".to_string()))?;

    u64::try_from(total).map_err(|_| InternalError::Internal("fee_limit exceeds u64".to_string()))
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
    _project_id: &str,
) -> Result<TransactionRpc, BuildPosTxsError> {
    let to_eth = tron_base58_to_eth_address(&params.recipient_address).map_err(|e| {
        BuildPosTxsError::Validation(ValidationError::InvalidRecipient(e.to_string()))
    })?;
    let decimals = fetch_trc20_decimals(
        state.clone(),
        params.asset.chain_id(),
        &params.sender_address,
        params.asset.asset_reference(),
    )
    .await?;
    let token_amount = parse_token_amount(&params.amount, decimals)?;

    let data = transferCall {
        to: to_eth,
        value: token_amount,
    }
    .abi_encode();

    let params_hex = &hex::encode(&data[4..]);

    let owner_address = tron_b58_to_hex41(&params.sender_address)
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidSender(e.to_string())))?;
    let contract_address = tron_b58_to_hex41(params.asset.asset_reference())
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string())))?;

    let trigger_req = TriggerSmartContractRequest {
        owner_address,
        contract_address,
        function_selector: "transfer(address,uint256)".to_string(),
        parameter: params_hex.to_string(),
        fee_limit: None,
        call_value: 0,
        visible: false,
    };

    let fee_limit =
        estimate_trc20_fee_limit(state.clone(), params.asset.chain_id(), &trigger_req).await?;

    let trigger_req = TriggerSmartContractRequest {
        fee_limit: Some(fee_limit),
        ..trigger_req
    };

    let resp = trigger_smart_contract(state, params.asset.chain_id(), &trigger_req)
        .await
        .map_err(BuildPosTxsError::Internal)?;

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
    state: State<Arc<AppState>>,
    chain_id: &Caip2ChainId,
    owner_b58: &str,
    contract_b58: &str,
) -> Result<u8, BuildPosTxsError> {
    let owner_address = tron_b58_to_hex41(owner_b58)
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string())))?;
    let contract_address = tron_b58_to_hex41(contract_b58)
        .map_err(|e| BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string())))?;

    let trigger_req = TriggerConstantContractRequest {
        owner_address,
        contract_address,
        function_selector: "decimals()".to_string(),
        parameter: String::new(),
        visible: false,
    };

    let resp = trigger_constant_contract(state, chain_id, &trigger_req)
        .await
        .map_err(BuildPosTxsError::Internal)?;

    if let Some(results) = resp.constant_result {
        if let Some(hex_str) = results.first() {
            let bytes = hex::decode(hex_str).map_err(|e| {
                BuildPosTxsError::Internal(InternalError::Internal(format!(
                    "Failed to decode decimals result: {}",
                    e
                )))
            })?;
            if bytes.is_empty() {
                return Err(BuildPosTxsError::Internal(InternalError::Internal(
                    "Empty decimals result".to_string(),
                )));
            }
            let decimals = *bytes.last().unwrap() as u8;
            return Ok(decimals);
        }
    }

    Err(BuildPosTxsError::Internal(InternalError::Internal(
        "Missing decimals in response".to_string(),
    )))
}

pub async fn get_transaction_status(
    state: State<Arc<AppState>>,
    _project_id: &str,
    signed_tx: &SignedTransaction,
    chain_id: &Caip2ChainId,
) -> Result<TransactionStatus, CheckPosTxError> {
    let txid = signed_tx.tx_id.as_str();

    let already_broadcasted = get_transaction_by_id(state.clone(), chain_id, txid)
        .await
        .map_err(CheckPosTxError::Internal)?
        .is_some();

    if !already_broadcasted {
        let broadcast_resp = broadcast_transaction(state, chain_id, signed_tx)
            .await
            .map_err(CheckPosTxError::Internal)?;
        debug!("tron broadcast resp: {:?}", broadcast_resp);
        if broadcast_resp.result.is_none() || broadcast_resp.result == Some(false) {
            return Err(CheckPosTxError::Internal(InternalError::Internal(format!(
                "Broadcast failed: {} {}",
                broadcast_resp.code.unwrap_or_default(),
                broadcast_resp.message.unwrap_or_default()
            ))));
        }

        return Ok(TransactionStatus::Pending);
    }

    let info_opt = get_transaction_info_by_id(state, chain_id, txid)
        .await
        .map_err(CheckPosTxError::Internal)?;

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

    let status = get_transaction_status(state, project_id, &signed_tx, chain_id).await?;

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
