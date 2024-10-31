use {
    super::{super::HANDLER_TASK_METRICS, check_bridging_for_erc20_transfer},
    crate::{
        analytics::MessageSource,
        error::RpcError,
        state::AppState,
        utils::crypto::{
            convert_alloy_address_to_h160, decode_erc20_function_type, decode_erc20_transfer_data,
            get_balance, get_erc20_balance, Erc20FunctionType,
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

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<QueryParams>,
    Json(request_payload): Json<CheckTransactionRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("ca_check"))
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
    let transfer_value_string = request_payload.transaction.value;
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
            return Ok(Json(RequiresMultiChainResponse {
                requires_multi_chain: false,
            })
            .into_response());
        }
    }

    // Check if the ERC20 function is the `transfer` function
    let transaction_data = hex::decode(request_payload.transaction.data.trim_start_matches("0x"))
        .map_err(|e| RpcError::WrongHexFormat(e.to_string()))?;
    if decode_erc20_function_type(&transaction_data)? != Erc20FunctionType::Transfer {
        error!("The transaction data is not a transfer function");
        return Ok(Json(RequiresMultiChainResponse {
            requires_multi_chain: false,
        })
        .into_response());
    }

    // Decode the ERC20 transfer function data
    let (_erc20_receiver, erc20_transfer_value) = decode_erc20_transfer_data(&transaction_data)?;

    // Get the current balance of the ERC20 token and check if it's enough for the transfer
    // without bridging or calculate the top-up value
    let erc20_balance = get_erc20_balance(
        &request_payload.transaction.chain_id,
        convert_alloy_address_to_h160(to_address),
        convert_alloy_address_to_h160(from_address),
        &query_params.project_id,
        MessageSource::ChainAgnosticCheck,
    )
    .await?;
    let erc20_balance = U256::from_be_bytes(erc20_balance.into());
    if erc20_balance >= erc20_transfer_value {
        // The balance is sufficient for the transfer no need for bridging
        return Ok(Json(RequiresMultiChainResponse {
            requires_multi_chain: false,
        })
        .into_response());
    }
    let erc20_topup_value = erc20_transfer_value - erc20_balance;

    // Check for possible bridging by iterating over supported assets
    if let Some((bridge_chain_id, bridge_contract)) =
        check_bridging_for_erc20_transfer(query_params.project_id, erc20_topup_value, from_address)
            .await?
    {
        // Skip bridging if that's the same chainId and contract address
        if bridge_chain_id == request_payload.transaction.chain_id && bridge_contract == to_address
        {
            return Ok(Json(RequiresMultiChainResponse {
                requires_multi_chain: false,
            })
            .into_response());
        }

        return Ok(Json(RequiresMultiChainResponse {
            requires_multi_chain: true,
        })
        .into_response());
    }

    // No sufficient balances found for the transfer or bridging
    Ok(Json(RequiresMultiChainResponse {
        requires_multi_chain: false,
    })
    .into_response())
}
