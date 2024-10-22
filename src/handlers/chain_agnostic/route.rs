use {
    super::{super::HANDLER_TASK_METRICS, check_bridging_for_erc20_transfer},
    crate::{
        analytics::MessageSource,
        error::RpcError,
        state::AppState,
        utils::crypto::{
            convert_alloy_address_to_h160, decode_erc20_function_type, decode_erc20_transfer_data,
            get_balance, Erc20FunctionType,
        },
    },
    alloy::primitives::{Address, U256},
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::{str::FromStr, sync::Arc},
    tracing::{debug, error},
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
    from: String,
    to: String,
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
    orchestration_id: String,
    transactions: Vec<Transaction>,
}

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

    let from_address = Address::from_str(&request_payload.transaction.from)
        .map_err(|_| RpcError::InvalidAddress)?;
    let to_address =
        Address::from_str(&request_payload.transaction.to).map_err(|_| RpcError::InvalidAddress)?;

    // Check the native token balance
    let native_token_balance = get_balance(
        &request_payload.transaction.chain_id,
        convert_alloy_address_to_h160(from_address),
        &query_params.project_id.clone(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;
    let transfer_value_string = request_payload.transaction.value.clone();
    let transfer_value = U256::from_str(&transfer_value_string)
        .map_err(|_| RpcError::InvalidValue(transfer_value_string))?;

    // check if the transaction value is non zero so it's a native token transfer
    if transfer_value > U256::ZERO {
        debug!(
            "The transaction is a native token transfer with value: {:?}",
            transfer_value
        );
        // If the native token balance is greater than the transfer value, we don't need multi-chain bridging
        if U256::from_be_bytes(native_token_balance.into()) > transfer_value {
            return Err(RpcError::NoBridgingNeeded);
        }
    }

    // Check if the ERC20 function is the `transfer` function
    let transaction_data = hex::decode(request_payload.transaction.data.trim_start_matches("0x"))
        .map_err(|e| RpcError::WrongHexFormat(e.to_string()))?;
    if decode_erc20_function_type(&transaction_data)? != Erc20FunctionType::Transfer {
        error!("The transaction data is not a transfer function");
        return Err(RpcError::NoBridgingNeeded);
    }

    // Decode the ERC20 transfer function data
    let (_erc20_receiver, erc20_transfer_value) = decode_erc20_transfer_data(&transaction_data)?;

    // Check for possible bridging by iterating over supported assets
    if let Some((bridge_chain_id, bridge_contract)) = check_bridging_for_erc20_transfer(
        query_params.project_id,
        erc20_transfer_value,
        from_address,
    )
    .await?
    {
        // Skip bridging if that's the same chainId and contract address
        if bridge_chain_id == request_payload.transaction.chain_id && bridge_contract == to_address
        {
            return Err(RpcError::NoBridgingNeeded);
        }

        // Get Quotes for the bridging
        let quotes = state
            .providers
            .chain_orchestrator_provider
            .get_bridging_quotes(
                bridge_chain_id,
                bridge_contract,
                request_payload.transaction.chain_id.clone(),
                to_address,
                erc20_transfer_value,
                from_address,
            )
            .await?;

        // Build bridging transaction
        let best_route = quotes.first().ok_or(RpcError::NoBridgingRoutesAvailable)?;
        let bridge_tx = state
            .providers
            .chain_orchestrator_provider
            .build_bridging_tx(best_route.clone())
            .await?;

        // Return the bridging transactions
        let orchestration_id = Uuid::new_v4().to_string();

        let mut routes = Vec::new();
        routes.push(Transaction {
            from: from_address.to_string(),
            to: bridge_tx.tx_target,
            value: bridge_tx.value,
            gas_price: request_payload.transaction.gas_price.clone(),
            gas: request_payload.transaction.gas.clone(),
            data: bridge_tx.tx_data,
            nonce: request_payload.transaction.nonce.clone(),
            max_fee_per_gas: request_payload.transaction.max_fee_per_gas.clone(),
            max_priority_fee_per_gas: request_payload.transaction.max_priority_fee_per_gas.clone(),
            chain_id: format!("eip155:{}", bridge_tx.chain_id),
        });
        // Push initial transaction last after bridging transactions
        routes.push(request_payload.transaction);

        return Ok(Json(RouteResponse {
            orchestration_id,
            transactions: routes,
        })
        .into_response());
    }

    Err(RpcError::NoBridgingNeeded)
}
