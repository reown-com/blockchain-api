use {
    super::{super::HANDLER_TASK_METRICS, check_erc20_balances, BRIDGING_AVAILABLE_ASSETS},
    crate::{
        analytics::MessageSource,
        error::RpcError,
        state::AppState,
        utils::crypto::{
            convert_alloy_address_to_h160, decode_erc20_call_function_data, get_balance,
            Erc20FunctionType,
        },
    },
    alloy::primitives::{Address, U256},
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, str::FromStr, sync::Arc},
    tracing::debug,
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

    // If the native token balance is greater than the transfer value, we don't need multi-chain bridging
    if U256::from_be_bytes(native_token_balance.into()) > transfer_value {
        return Ok(Json(RequiresMultiChainResponse {
            requires_multi_chain: false,
        })
        .into_response());
    }

    // Check if the transaction data is the `transfer`` ERC20 function
    let transaction_data = hex::decode(request_payload.transaction.data.trim_start_matches("0x"))
        .map_err(|e| RpcError::WrongHexFormat(e.to_string()))?;
    if decode_erc20_call_function_data(&transaction_data)? != Erc20FunctionType::Transfer {
        debug!("The transaction data is not a transfer function");
        return Ok(Json(RequiresMultiChainResponse {
            requires_multi_chain: false,
        })
        .into_response());
    }

    // Check the ERC20 tokens balance for each of supported assets
    let mut contracts_per_chain: HashMap<String, Vec<String>> = HashMap::new();
    for (_, chain_map) in BRIDGING_AVAILABLE_ASSETS.entries() {
        for (chain_id, contract_address) in chain_map.entries() {
            contracts_per_chain
                .entry((*chain_id).to_string())
                .or_default()
                .push((*contract_address).to_string());
        }
    }
    // Making the check for each chain_id
    for (chain_id, contracts) in contracts_per_chain {
        let erc20_balances = check_erc20_balances(
            query_params.project_id.clone(),
            from_address,
            chain_id.clone(),
            contracts
                .iter()
                .map(|c| Address::from_str(c).unwrap())
                .collect(),
        )
        .await?;
        for (contract, balance) in erc20_balances {
            if balance > transfer_value {
                debug!(
                    "The ERC20 balance of {:?} is sufficient for the transfer",
                    contract
                );
                return Ok(Json(RequiresMultiChainResponse {
                    requires_multi_chain: true,
                })
                .into_response());
            }
        }
    }

    // No balance is sufficient for the transfer or bridging
    Ok(Json(RequiresMultiChainResponse {
        requires_multi_chain: false,
    })
    .into_response())
}
