use {
    super::{
        super::HANDLER_TASK_METRICS, check_bridging_for_erc20_transfer, BridgingStatus,
        StorageBridgingItem, BRIDGING_AMOUNT_MULTIPLIER,
    },
    crate::{
        analytics::MessageSource,
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::crypto::{
            convert_alloy_address_to_h160, decode_erc20_function_type, decode_erc20_transfer_data,
            get_balance, get_erc20_balance, get_gas_price, get_nonce, Erc20FunctionType,
        },
    },
    alloy::primitives::{Address, U256},
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::{
        str::FromStr,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    },
    tracing::debug,
    uuid::Uuid,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub project_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckTransactionRequest {
    transaction: Transaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    from: Address,
    to: Address,
    value: String,
    gas: String,
    gas_price: String,
    data: String,
    nonce: String,
    max_fee_per_gas: String,
    max_priority_fee_per_gas: String,
    chain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequiresMultiChainResponse {
    requires_multi_chain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    orchestration_id: Option<String>,
    transactions: Vec<Transaction>,
    metadata: Option<Metadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    funding_from: Vec<FundingMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingMetadata {
    chain_id: String,
    token_contract: Address,
    symbol: String,
    amount: String,
}

/// Bridging check error response that should be returned as a normal HTTP 200 response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    error: BridgingError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgingError {
    NoRoutesAvailable,
    InsufficientFunds,
    InsufficientGasFunds,
}

const NO_BRIDING_NEEDED_RESPONSE: Json<RouteResponse> = Json(RouteResponse {
    transactions: vec![],
    orchestration_id: None,
    metadata: None,
});

// Default gas estimate
// Using default with 6x increase
const DEFAULT_GAS: i64 = 0x029a6b * 0x6;

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<QueryParams>,
    Json(request_payload): Json<CheckTransactionRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("ca_route"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Query(query_params): Query<QueryParams>,
    request_payload: CheckTransactionRequest,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query_params.project_id.clone())
        .await?;

    let from_address = request_payload.transaction.from;
    let to_address = request_payload.transaction.to;
    let transfer_value_string = request_payload.transaction.value.clone();
    let transfer_value = U256::from_str(&transfer_value_string)
        .map_err(|_| RpcError::InvalidValue(transfer_value_string))?;

    // Check if the transaction value is non zero it's a native token transfer
    if transfer_value > U256::ZERO {
        debug!(
            "The transaction is a native token transfer with value: {:?}",
            transfer_value
        );
        return Ok(NO_BRIDING_NEEDED_RESPONSE.into_response());
    }

    // Check if the ERC20 function is the `transfer` function
    let transaction_data = hex::decode(request_payload.transaction.data.trim_start_matches("0x"))
        .map_err(|e| RpcError::WrongHexFormat(e.to_string()))?;
    if decode_erc20_function_type(&transaction_data)? != Erc20FunctionType::Transfer {
        debug!("The transaction data is not a transfer function");
        return Ok(NO_BRIDING_NEEDED_RESPONSE.into_response());
    }

    // Decode the ERC20 transfer function data
    let (_erc20_receiver, erc20_transfer_value) = decode_erc20_transfer_data(&transaction_data)?;

    // Get the current balance of the ERC20 token and check if it's enough for the transfer
    // without bridging or calculate the top-up value
    let erc20_balance = get_erc20_balance(
        &request_payload.transaction.chain_id,
        convert_alloy_address_to_h160(to_address),
        convert_alloy_address_to_h160(from_address),
        &query_params.project_id.clone(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;
    let erc20_balance = U256::from_be_bytes(erc20_balance.into());
    if erc20_balance >= erc20_transfer_value {
        return Ok(NO_BRIDING_NEEDED_RESPONSE.into_response());
    }

    // Multiply the topup value by the bridging percent multiplier
    let erc20_topup_value = erc20_transfer_value - erc20_balance;
    let erc20_topup_value = erc20_topup_value
        + (erc20_topup_value * U256::from(BRIDGING_AMOUNT_MULTIPLIER)) / U256::from(100);

    // Check for possible bridging funds by iterating over supported assets
    // or return an insufficient funds error
    let Some((bridge_chain_id, bridge_token_symbol, bridge_contract)) =
        check_bridging_for_erc20_transfer(
            query_params.project_id.clone(),
            erc20_topup_value,
            from_address,
        )
        .await?
    else {
        return Ok(Json(ErrorResponse {
            error: BridgingError::InsufficientFunds,
        })
        .into_response());
    };

    // Skip bridging if that's the same chainId and contract address
    if bridge_chain_id == request_payload.transaction.chain_id && bridge_contract == to_address {
        return Ok(NO_BRIDING_NEEDED_RESPONSE.into_response());
    }

    // Check the native token balance for the bridging chainId for gas fees paying
    let native_token_balance = get_balance(
        &bridge_chain_id,
        convert_alloy_address_to_h160(from_address),
        &query_params.project_id.clone(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;
    if U256::from_be_bytes(native_token_balance.into()) == U256::ZERO {
        return Ok(Json(ErrorResponse {
            error: BridgingError::InsufficientGasFunds,
        })
        .into_response());
    }

    // Get Quotes for the bridging
    let quotes = state
        .providers
        .chain_orchestrator_provider
        .get_bridging_quotes(
            bridge_chain_id.clone(),
            bridge_contract,
            request_payload.transaction.chain_id.clone(),
            to_address,
            erc20_topup_value,
            from_address,
        )
        .await?;

    // Build bridging transaction
    let mut routes = Vec::new();
    let Some(best_route) = quotes.first() else {
        return Ok(Json(ErrorResponse {
            error: BridgingError::NoRoutesAvailable,
        })
        .into_response());
    };
    let bridge_tx = state
        .providers
        .chain_orchestrator_provider
        .build_bridging_tx(best_route.clone())
        .await?;

    // Getting the current nonce for the address
    let mut current_nonce = get_nonce(
        format!("eip155:{}", bridge_tx.chain_id).as_str(),
        from_address,
        &query_params.project_id.clone(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;

    // Getting the current gas price
    let gas_price = get_gas_price(
        &bridge_chain_id.clone(),
        &query_params.project_id.clone(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;

    // TODO: Implement gas estimation using `eth_estimateGas` for each transaction

    // Check for the allowance
    if let Some(approval_data) = bridge_tx.approval_data {
        let allowance = state
            .providers
            .chain_orchestrator_provider
            .check_allowance(
                format!("eip155:{}", bridge_tx.chain_id),
                approval_data.owner,
                approval_data.allowance_target,
                approval_data.approval_token_address,
            )
            .await?;

        // Check if the approval transaction injection is needed
        if approval_data.minimum_approval_amount >= allowance {
            let approval_tx = state
                .providers
                .chain_orchestrator_provider
                .build_approval_tx(
                    format!("eip155:{}", bridge_tx.chain_id),
                    approval_data.owner,
                    approval_data.allowance_target,
                    approval_data.approval_token_address,
                    erc20_topup_value,
                )
                .await?;

            routes.push(Transaction {
                from: approval_tx.from,
                to: approval_tx.to,
                value: "0x00".to_string(),
                gas_price: format!("0x{:x}", gas_price),
                gas: format!("0x{:x}", DEFAULT_GAS),
                data: approval_tx.data,
                nonce: format!("0x{:x}", current_nonce),
                max_fee_per_gas: request_payload.transaction.max_fee_per_gas.clone(),
                max_priority_fee_per_gas: request_payload
                    .transaction
                    .max_priority_fee_per_gas
                    .clone(),
                chain_id: format!("eip155:{}", bridge_tx.chain_id),
            });
            current_nonce += 1;
        }
    }

    // Push bridging transaction
    routes.push(Transaction {
        from: from_address,
        to: bridge_tx.tx_target,
        value: bridge_tx.value,
        gas_price: format!("0x{:x}", gas_price),
        gas: format!("0x{:x}", DEFAULT_GAS),
        data: bridge_tx.tx_data,
        nonce: format!("0x{:x}", current_nonce),
        max_fee_per_gas: request_payload.transaction.max_fee_per_gas.clone(),
        max_priority_fee_per_gas: request_payload.transaction.max_priority_fee_per_gas.clone(),
        chain_id: format!("eip155:{}", bridge_tx.chain_id),
    });

    // Save the bridging transaction to the IRN
    let orchestration_id = Uuid::new_v4().to_string();
    let bridging_status_item = StorageBridgingItem {
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
        chain_id: request_payload.transaction.chain_id,
        wallet: from_address,
        contract: to_address,
        amount_expected: erc20_transfer_value, // The total transfer amount expected
        status: BridgingStatus::Pending,
        error_reason: None,
    };
    let irn_client = state.irn.as_ref().ok_or(RpcError::IrnNotConfigured)?;
    let irn_call_start = SystemTime::now();
    irn_client
        .set(
            orchestration_id.clone(),
            serde_json::to_string(&bridging_status_item)?.into(),
        )
        .await?;
    state
        .metrics
        .add_irn_latency(irn_call_start, OperationType::Set);

    return Ok(Json(RouteResponse {
        orchestration_id: Some(orchestration_id),
        transactions: routes,
        metadata: Some(Metadata {
            funding_from: vec![FundingMetadata {
                chain_id: bridge_chain_id,
                token_contract: bridge_contract,
                symbol: bridge_token_symbol,
                amount: format!("0x{:x}", erc20_topup_value),
            }],
        }),
    })
    .into_response());
}
