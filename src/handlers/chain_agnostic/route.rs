use {
    super::{
        super::HANDLER_TASK_METRICS, check_bridging_for_erc20_transfer,
        find_supported_bridging_asset, get_assets_changes_from_simulation, BridgingStatus,
        StorageBridgingItem, BRIDGING_AMOUNT_SLIPPAGE, STATUS_POLLING_INTERVAL,
    },
    crate::{
        analytics::MessageSource,
        error::RpcError,
        state::AppState,
        storage::irn::OperationType,
        utils::crypto::{
            convert_alloy_address_to_h160, decode_erc20_transfer_data, get_erc20_balance, get_nonce,
        },
    },
    alloy::primitives::{Address, U256, U64},
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
    tracing::{debug, error},
    uuid::Uuid,
    wc::future::FutureExt,
    yttrium::chain_abstraction::api::{
        route::{
            BridgingError, FundingMetadata, InitialTransactionMetadata, Metadata, RouteQueryParams,
            RouteRequest, RouteResponse, RouteResponseAvailable, RouteResponseError,
            RouteResponseNotRequired, RouteResponseSuccess,
        },
        Transaction,
    },
};

// Default gas estimate
// Using default with 6x increase
const DEFAULT_GAS: i64 = 0x029a6b * 0x6;

// Slippage for the gas estimation
const ESTIMATED_GAS_SLIPPAGE: i8 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteRoute {
    pub to_amount: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<RouteQueryParams>,
    Json(request_payload): Json<RouteRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("ca_route"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    Query(query_params): Query<RouteQueryParams>,
    request_payload: RouteRequest,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(query_params.project_id.as_ref())
        .await?;

    let mut initial_transaction = Transaction {
        from: request_payload.transaction.from,
        to: request_payload.transaction.to,
        value: request_payload.transaction.value,
        gas_limit: U64::from(DEFAULT_GAS),
        input: request_payload.transaction.input.clone(),
        nonce: U64::ZERO,
        chain_id: request_payload.transaction.chain_id.clone(),
    };

    let from_address = initial_transaction.from;
    let to_address = initial_transaction.to;
    let transfer_value = initial_transaction.value;

    // Calculate the initial transaction nonce
    let intial_transaction_nonce = get_nonce(
        &initial_transaction.chain_id.clone(),
        from_address,
        query_params.project_id.as_ref(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;
    initial_transaction.nonce = intial_transaction_nonce;

    let no_bridging_needed_response: Json<RouteResponse> = Json(RouteResponse::Success(
        RouteResponseSuccess::NotRequired(RouteResponseNotRequired {
            initial_transaction: initial_transaction.clone(),
            transactions: vec![],
        }),
    ));

    // Check if the transaction value is non zero it's a native token transfer
    if transfer_value > U256::ZERO {
        debug!(
            "The transaction is a native token transfer with value: {:?}",
            transfer_value
        );
        return Ok(no_bridging_needed_response.into_response());
    }
    let transaction_data = initial_transaction.input.clone();

    // Decode the ERC20 transfer function data or use the simulation
    // to get the transfer asset and amount
    let (asset_transfer_contract, asset_transfer_value, asset_transfer_receiver, gas_used) =
        match decode_erc20_transfer_data(&transaction_data) {
            Ok((receiver, erc20_transfer_value)) => {
                debug!(
                    "The transaction is an ERC20 transfer with value: {:?}",
                    erc20_transfer_value
                );

                // Check if the destination address is supported ERC20 asset contract
                if find_supported_bridging_asset(
                    &request_payload.transaction.chain_id.clone(),
                    to_address,
                )
                .is_none()
                {
                    error!("The destination address is not a supported bridging asset contract");
                    return Ok(no_bridging_needed_response.into_response());
                };

                // Get the ERC20 transfer gas estimation for the token contract
                // and chain_id, or simulate the transaction to get the gas used
                let gas_used = match state
                    .providers
                    .simulation_provider
                    .get_cached_erc20_gas_estimation(
                        &request_payload.transaction.chain_id,
                        to_address,
                    )
                    .await?
                {
                    Some(gas) => gas,
                    None => {
                        let (_, simulated_gas_used) = get_assets_changes_from_simulation(
                            state.providers.simulation_provider.clone(),
                            &initial_transaction,
                            state.metrics.clone(),
                        )
                        .await?;
                        state
                            .providers
                            .simulation_provider
                            .set_cached_erc20_gas_estimation(
                                &request_payload.transaction.chain_id,
                                to_address,
                                simulated_gas_used,
                            )
                            .await?;
                        simulated_gas_used
                    }
                };

                (to_address, erc20_transfer_value, receiver, gas_used)
            }
            _ => {
                debug!(
                    "The transaction data is not an ERC20 transfer function, making a simulation"
                );

                let (simulation_assets_changes, gas_used) = get_assets_changes_from_simulation(
                    state.providers.simulation_provider.clone(),
                    &initial_transaction,
                    state.metrics.clone(),
                )
                .await?;
                let mut asset_transfer_value = U256::ZERO;
                let mut asset_transfer_contract = Address::default();
                let mut asset_transfer_receiver = Address::default();
                for asset_change in simulation_assets_changes {
                    if find_supported_bridging_asset(
                        &asset_change.chain_id.clone(),
                        asset_change.asset_contract,
                    )
                    .is_some()
                    {
                        asset_transfer_contract = asset_change.asset_contract;
                        asset_transfer_value = asset_change.amount;
                        asset_transfer_receiver = asset_change.receiver;
                        break;
                    }
                }
                if asset_transfer_value.is_zero() {
                    error!("The transaction does not change any supported bridging assets");
                    return Ok(no_bridging_needed_response.into_response());
                }

                (
                    asset_transfer_contract,
                    asset_transfer_value,
                    asset_transfer_receiver,
                    gas_used,
                )
            }
        };
    // Estimated gas multiplied by the slippage
    initial_transaction.gas_limit =
        U64::from((gas_used * (100 + ESTIMATED_GAS_SLIPPAGE as u64)) / 100);

    // Check if the destination address is supported ERC20 asset contract
    // Attempt to destructure the result into symbol and decimals using a match expression
    let (initial_tx_token_symbol, initial_tx_token_decimals) =
        match find_supported_bridging_asset(&request_payload.transaction.chain_id, to_address) {
            Some((symbol, decimals)) => (symbol, decimals),
            None => {
                error!("The destination address is not a supported bridging asset contract");
                return Ok(no_bridging_needed_response.into_response());
            }
        };

    // Get the current balance of the ERC20 token and check if it's enough for the transfer
    // without bridging or calculate the top-up value
    let erc20_balance = get_erc20_balance(
        &request_payload.transaction.chain_id,
        convert_alloy_address_to_h160(asset_transfer_contract),
        convert_alloy_address_to_h160(from_address),
        query_params.project_id.as_ref(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;
    let erc20_balance = U256::from_be_bytes(erc20_balance.into());
    if erc20_balance >= asset_transfer_value {
        return Ok(no_bridging_needed_response.into_response());
    }
    let erc20_topup_value = asset_transfer_value - erc20_balance;

    // Check for possible bridging funds by iterating over supported assets
    // or return an insufficient funds error
    let Some(bridging_asset) = check_bridging_for_erc20_transfer(
        query_params.project_id.as_ref().to_string(),
        erc20_topup_value,
        from_address,
        initial_transaction.chain_id.clone(),
        asset_transfer_contract,
    )
    .await?
    else {
        return Ok(Json(RouteResponse::Error(RouteResponseError {
            error: BridgingError::InsufficientFunds,
        }))
        .into_response());
    };
    let bridge_chain_id = bridging_asset.chain_id;
    let bridge_token_symbol = bridging_asset.token_symbol;
    let bridge_contract = bridging_asset.contract_address;
    let bridge_decimals = bridging_asset.decimals;
    let current_bridging_asset_balance = bridging_asset.current_balance;

    // Get Quotes for the bridging
    let quotes = state
        .providers
        .chain_orchestrator_provider
        .get_bridging_quotes(
            bridge_chain_id.clone(),
            bridge_contract,
            request_payload.transaction.chain_id.clone(),
            asset_transfer_contract,
            erc20_topup_value,
            from_address,
            state.metrics.clone(),
        )
        .await?;
    let Some(best_route) = quotes.first() else {
        return Ok(Json(RouteResponse::Error(RouteResponseError {
            error: BridgingError::NoRoutesAvailable,
        }))
        .into_response());
    };

    // Calculate the bridging fee based on the amount given from quotes
    let bridging_amount = serde_json::from_value::<QuoteRoute>(best_route.clone())?.to_amount;
    let bridging_amount =
        U256::from_str(&bridging_amount).map_err(|_| RpcError::InvalidValue(bridging_amount))?;
    let bridging_fee = erc20_topup_value - bridging_amount;

    // Calculate the required bridging topup amount with the bridging fee and slippage
    let required_topup_amount = erc20_topup_value + bridging_fee;
    let required_topup_amount = ((required_topup_amount * U256::from(BRIDGING_AMOUNT_SLIPPAGE))
        / U256::from(100))
        + required_topup_amount;
    if current_bridging_asset_balance < required_topup_amount {
        error!(
            "The current bridging asset balance on {} is {} less than the required topup amount:{}. The bridging fee is:{}",
            from_address, current_bridging_asset_balance, required_topup_amount, bridging_fee
        );
        return Ok(Json(RouteResponse::Error(RouteResponseError {
            error: BridgingError::InsufficientFunds,
        }))
        .into_response());
    }

    // Get quotes for updated topup amount
    let quotes = state
        .providers
        .chain_orchestrator_provider
        .get_bridging_quotes(
            bridge_chain_id.clone(),
            bridge_contract,
            request_payload.transaction.chain_id.clone(),
            asset_transfer_contract,
            required_topup_amount,
            from_address,
            state.metrics.clone(),
        )
        .await?;
    let Some(best_route) = quotes.first() else {
        return Ok(Json(RouteResponse::Error(RouteResponseError {
            error: BridgingError::NoRoutesAvailable,
        }))
        .into_response());
    };
    // Check the final bridging amount from the quote
    let bridging_amount = serde_json::from_value::<QuoteRoute>(best_route.clone())?.to_amount;
    let bridging_amount =
        U256::from_str(&bridging_amount).map_err(|_| RpcError::InvalidValue(bridging_amount))?;
    if erc20_topup_value > bridging_amount {
        error!(
            "The final bridging amount:{} is less than the topup amount:{}",
            bridging_amount, erc20_topup_value
        );
        return Err(RpcError::BridgingFinalAmountLess);
    }
    let final_bridging_fee = bridging_amount - erc20_topup_value;

    // Build bridging transaction
    let bridge_tx = state
        .providers
        .chain_orchestrator_provider
        .build_bridging_tx(best_route.clone(), state.metrics.clone())
        .await?;

    // Getting the current nonce for the address
    let mut current_nonce = get_nonce(
        format!("eip155:{}", bridge_tx.chain_id).as_str(),
        from_address,
        query_params.project_id.as_ref(),
        MessageSource::ChainAgnosticCheck,
    )
    .await?;

    // TODO: Implement gas estimation using `eth_estimateGas` for each transaction
    let mut routes = Vec::new();

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
                state.metrics.clone(),
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
                    required_topup_amount,
                    state.metrics.clone(),
                )
                .await?;

            routes.push(Transaction {
                from: approval_tx.from,
                to: approval_tx.to,
                value: U256::ZERO,
                gas_limit: U64::from(DEFAULT_GAS),
                input: approval_tx.data,
                nonce: current_nonce,
                chain_id: format!("eip155:{}", bridge_tx.chain_id),
            });
            current_nonce += U64::from(1);
        }
    }

    // Push bridging transaction
    routes.push(Transaction {
        from: from_address,
        to: bridge_tx.tx_target,
        value: bridge_tx.value,
        gas_limit: U64::from(DEFAULT_GAS),
        input: bridge_tx.tx_data,
        nonce: current_nonce,
        chain_id: format!("eip155:{}", bridge_tx.chain_id),
    });

    // Save the bridging transaction to the IRN
    let orchestration_id = Uuid::new_v4().to_string();
    let bridging_status_item = StorageBridgingItem {
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        chain_id: request_payload.transaction.chain_id,
        wallet: from_address,
        contract: to_address,
        amount_current: erc20_balance, // The current balance of the ERC20 token
        amount_expected: asset_transfer_value, // The total transfer amount expected
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

    return Ok(Json(RouteResponse::Success(RouteResponseSuccess::Available(
        RouteResponseAvailable {
            orchestration_id,
            initial_transaction,
            transactions: routes,
            metadata: Metadata {
                funding_from: vec![FundingMetadata {
                    chain_id: bridge_chain_id,
                    token_contract: bridge_contract,
                    symbol: bridge_token_symbol,
                    amount: bridging_amount,
                    bridging_fee: final_bridging_fee,
                    decimals: bridge_decimals,
                }],
                check_in: STATUS_POLLING_INTERVAL,
                initial_transaction: InitialTransactionMetadata {
                    transfer_to: asset_transfer_receiver,
                    amount: asset_transfer_value,
                    token_contract: to_address,
                    symbol: initial_tx_token_symbol,
                    decimals: initial_tx_token_decimals,
                },
            },
        },
    )))
    .into_response());
}
