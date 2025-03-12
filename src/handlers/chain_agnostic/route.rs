use {
    super::{
        super::HANDLER_TASK_METRICS, check_bridging_for_erc20_transfer, convert_amount,
        find_supported_bridging_asset, get_assets_changes_from_simulation, BridgingStatus,
        StorageBridgingItem, BRIDGING_FEE_SLIPPAGE, STATUS_POLLING_INTERVAL,
    },
    crate::{
        analytics::{
            ChainAbstractionBridgingInfo, ChainAbstractionFundingInfo,
            ChainAbstractionInitialTxInfo, MessageSource,
        },
        error::RpcError,
        handlers::{self_provider, SdkInfoParams},
        metrics::{ChainAbstractionNoBridgingNeededType, ChainAbstractionTransactionType},
        state::AppState,
        storage::irn::OperationType,
        utils::{
            crypto::{
                convert_alloy_address_to_h160, decode_erc20_transfer_data, get_erc20_balance,
                get_nonce, Erc20FunctionType,
            },
            network,
        },
    },
    alloy::primitives::{Address, U256, U64},
    axum::{
        extract::{ConnectInfo, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        net::SocketAddr,
        str::FromStr,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    },
    tracing::{debug, error},
    uuid::Uuid,
    wc::future::FutureExt,
    yttrium::chain_abstraction::api::{
        prepare::{
            BridgingError, FundingMetadata, InitialTransactionMetadata, Metadata, PrepareRequest,
            PrepareResponse, PrepareResponseAvailable, PrepareResponseError,
            PrepareResponseNotRequired, PrepareResponseSuccess, RouteQueryParams,
        },
        Transaction,
    },
};

// Slippage for the gas estimation
const ESTIMATED_GAS_SLIPPAGE: i16 = 500; // Temporarily x5 slippage to cover the volatility

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteRoute {
    pub to_amount: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query_params: Query<RouteQueryParams>,
    Json(request_payload): Json<PrepareRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, headers, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("ca_route"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query_params): Query<RouteQueryParams>,
    request_payload: PrepareRequest,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(query_params.project_id.as_ref())
        .await?;

    let provider_pool = self_provider::SelfProviderPool {
        state: state.0.clone(),
        connect_info: connect_info.0,
        headers: headers.clone(),
        project_id: query_params.project_id.as_ref().into(),
        sdk_info: SdkInfoParams {
            st: query_params.sdk_type.clone(),
            sv: query_params.sdk_version.clone(),
        },
        session_id: query_params.session_id.clone(),
    };

    let first_call = if let Some(first) = request_payload.transaction.calls.into_calls().first() {
        first.clone()
    } else {
        return Err(RpcError::InvalidParameter(
            "The transaction calls are empty".to_string(),
        ));
    };

    let mut initial_transaction = Transaction {
        from: request_payload.transaction.from,
        to: first_call.to,
        value: first_call.value,
        input: first_call.input.clone(),
        gas_limit: U64::ZERO,
        nonce: U64::ZERO,
        chain_id: request_payload.transaction.chain_id.clone(),
    };
    let initial_tx_chain_id = request_payload.transaction.chain_id.clone();

    let from_address = initial_transaction.from;
    let to_address = initial_transaction.to;
    let transfer_value = initial_transaction.value;

    // Calculate the initial transaction nonce
    let intial_transaction_nonce = get_nonce(
        from_address,
        &provider_pool.get_provider(
            initial_transaction.chain_id.clone(),
            MessageSource::ChainAgnosticCheck,
        ),
    )
    .await?;
    initial_transaction.nonce = intial_transaction_nonce;

    let no_bridging_needed_response: Json<PrepareResponse> = Json(PrepareResponse::Success(
        PrepareResponseSuccess::NotRequired(PrepareResponseNotRequired {
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
        state
            .metrics
            .add_ca_no_bridging_needed(ChainAbstractionNoBridgingNeededType::NativeTokenTransfer);
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
                if find_supported_bridging_asset(&initial_tx_chain_id.clone(), to_address).is_none()
                {
                    error!(
                        "The destination address is not a supported bridging asset contract {}:{}",
                        initial_tx_chain_id.clone(),
                        to_address
                    );
                    state.metrics.add_ca_no_bridging_needed(
                        ChainAbstractionNoBridgingNeededType::AssetNotSupported,
                    );
                    return Ok(no_bridging_needed_response.into_response());
                };

                // Get the ERC20 transfer gas estimation for the token contract
                // and chain_id, or simulate the transaction to get the gas used
                let gas_used = match state
                    .providers
                    .simulation_provider
                    .get_cached_gas_estimation(
                        &initial_tx_chain_id,
                        to_address,
                        Some(Erc20FunctionType::Transfer),
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
                        state.metrics.add_ca_gas_estimation(
                            simulated_gas_used,
                            initial_transaction.chain_id.clone(),
                            ChainAbstractionTransactionType::Transfer,
                        );
                        // Save the initial tx gas estimation to the cache
                        {
                            let state = state.clone();
                            let initial_tx_chain_id = initial_tx_chain_id.clone();
                            tokio::spawn(async move {
                                state
                                    .providers
                                    .simulation_provider
                                    .set_cached_gas_estimation(
                                        &initial_tx_chain_id,
                                        to_address,
                                        Some(Erc20FunctionType::Transfer),
                                        simulated_gas_used,
                                    )
                                    .await
                                    .unwrap_or_else(|e| {
                                        error!(
                                "Failed to save the initial ERC20 gas estimation to the cache: {}",
                                e
                            )
                                    });
                            });
                        }
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
                    state.metrics.add_ca_no_bridging_needed(
                        ChainAbstractionNoBridgingNeededType::AssetNotSupported,
                    );
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
        match find_supported_bridging_asset(&initial_tx_chain_id, asset_transfer_contract) {
            Some((symbol, decimals)) => (symbol, decimals),
            None => {
                error!("The changed asset is not a supported for the bridging");
                state.metrics.add_ca_no_bridging_needed(
                    ChainAbstractionNoBridgingNeededType::AssetNotSupported,
                );
                return Ok(no_bridging_needed_response.into_response());
            }
        };

    // Get the current balance of the ERC20 token and check if it's enough for the transfer
    // without bridging or calculate the top-up value
    let erc20_balance = get_erc20_balance(
        &initial_tx_chain_id,
        convert_alloy_address_to_h160(asset_transfer_contract),
        convert_alloy_address_to_h160(from_address),
        query_params.project_id.as_ref(),
        MessageSource::ChainAgnosticCheck,
        query_params.session_id.clone(),
    )
    .await?;
    let erc20_balance = U256::from_be_bytes(erc20_balance.into());
    if erc20_balance >= asset_transfer_value {
        state
            .metrics
            .add_ca_no_bridging_needed(ChainAbstractionNoBridgingNeededType::SufficientFunds);
        return Ok(no_bridging_needed_response.into_response());
    }
    let mut erc20_topup_value = asset_transfer_value - erc20_balance;

    // Check for possible bridging funds by iterating over supported assets
    // or return an insufficient funds error
    let Some(bridging_asset) = check_bridging_for_erc20_transfer(
        query_params.project_id.to_string(),
        query_params.session_id.clone(),
        erc20_topup_value,
        from_address,
        initial_transaction.chain_id.clone(),
        asset_transfer_contract,
        initial_tx_token_symbol.clone(),
        initial_tx_token_decimals,
    )
    .await?
    else {
        state.metrics.add_ca_insufficient_funds();
        return Ok(Json(PrepareResponse::Error(PrepareResponseError {
            error: BridgingError::InsufficientFunds,
            reason: format!(
                "No supported assets with at least {} amount were found in the address {}",
                erc20_topup_value, from_address
            ),
        }))
        .into_response());
    };
    let bridge_chain_id = bridging_asset.chain_id;
    let bridge_token_symbol = bridging_asset.token_symbol;
    let bridge_contract = bridging_asset.contract_address;
    let bridge_decimals = bridging_asset.decimals;
    let current_bridging_asset_balance = bridging_asset.current_balance;

    // Applying decimals differences between initial token and bridging token
    erc20_topup_value = convert_amount(
        erc20_topup_value,
        initial_tx_token_decimals,
        bridge_decimals,
    );

    // Get Quotes for the bridging
    let quotes = state
        .providers
        .chain_orchestrator_provider
        .get_bridging_quotes(
            bridge_chain_id.clone(),
            bridge_contract,
            initial_tx_chain_id.clone(),
            asset_transfer_contract,
            erc20_topup_value,
            from_address,
            state.metrics.clone(),
        )
        .await?;
    let Some(best_route) = quotes.first() else {
        state
            .metrics
            .add_ca_no_routes_found(construct_metrics_bridging_route(
                bridge_chain_id.clone(),
                bridge_contract.to_string(),
                initial_tx_chain_id.clone(),
                asset_transfer_contract.to_string(),
            ));
        return Ok(Json(PrepareResponse::Error(PrepareResponseError {
            error: BridgingError::NoRoutesAvailable,
            reason: format!(
                "No routes were found from {}:{} to {}:{} for an initial amount {}",
                bridge_chain_id.clone(),
                bridge_contract,
                initial_tx_chain_id.clone(),
                asset_transfer_contract,
                erc20_topup_value
            ),
        }))
        .into_response());
    };

    // Calculate the bridging fee based on the amount given from quotes
    let bridging_amount = serde_json::from_value::<QuoteRoute>(best_route.clone())?.to_amount;
    let bridging_amount =
        U256::from_str(&bridging_amount).map_err(|_| RpcError::InvalidValue(bridging_amount))?;
    let bridging_fee = erc20_topup_value
        - convert_amount(bridging_amount, initial_tx_token_decimals, bridge_decimals);

    // Calculate the required bridging topup amount with the bridging fee
    // and bridging fee * slippage to cover volatility
    let required_topup_amount = erc20_topup_value + bridging_fee;
    let required_topup_amount = ((bridging_fee * U256::from(BRIDGING_FEE_SLIPPAGE))
        / U256::from(100))
        + required_topup_amount;
    if current_bridging_asset_balance < required_topup_amount {
        let error_reason = format!(
            "The current bridging asset balance on {} is {} less than the required topup amount:{}",
            from_address, current_bridging_asset_balance, required_topup_amount
        );
        error!(error_reason);
        state.metrics.add_ca_insufficient_funds();
        return Ok(Json(PrepareResponse::Error(PrepareResponseError {
            error: BridgingError::InsufficientFunds,
            reason: error_reason,
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
            initial_tx_chain_id.clone(),
            asset_transfer_contract,
            required_topup_amount,
            from_address,
            state.metrics.clone(),
        )
        .await?;
    let Some(best_route) = quotes.first() else {
        state
            .metrics
            .add_ca_no_routes_found(construct_metrics_bridging_route(
                bridge_chain_id.clone(),
                bridge_contract.to_string(),
                initial_tx_chain_id.clone(),
                asset_transfer_contract.to_string(),
            ));
        return Ok(Json(PrepareResponse::Error(PrepareResponseError {
            error: BridgingError::NoRoutesAvailable,
            reason: format!(
                "No routes were found from {}:{} to {}:{} for updated (fee included) amount: {}",
                bridge_chain_id.clone(),
                bridge_contract,
                initial_tx_chain_id.clone(),
                asset_transfer_contract,
                required_topup_amount
            ),
        }))
        .into_response());
    };

    // Check the final bridging amount from the quote
    let bridging_amount = serde_json::from_value::<QuoteRoute>(best_route.clone())?.to_amount;
    let bridging_amount =
        U256::from_str(&bridging_amount).map_err(|_| RpcError::InvalidValue(bridging_amount))?;
    let bridging_amount =
        convert_amount(bridging_amount, initial_tx_token_decimals, bridge_decimals);

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
        from_address,
        &provider_pool.get_provider(
            format!("eip155:{}", bridge_tx.chain_id),
            MessageSource::ChainAgnosticCheck,
        ),
    )
    .await?;

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

            let approval_transaction = Transaction {
                from: approval_tx.from,
                to: approval_tx.to,
                value: U256::ZERO,
                gas_limit: U64::ZERO,
                input: approval_tx.data,
                nonce: current_nonce,
                chain_id: format!("eip155:{}", bridge_tx.chain_id),
            };
            routes.push(approval_transaction);

            // Increment the nonce
            current_nonce += U64::from(1);
        }
    }

    let bridging_transaction = Transaction {
        from: from_address,
        to: bridge_tx.tx_target,
        value: bridge_tx.value,
        gas_limit: U64::ZERO,
        input: bridge_tx.tx_data,
        nonce: current_nonce,
        chain_id: format!("eip155:{}", bridge_tx.chain_id),
    };
    routes.push(bridging_transaction);

    // Estimate the gas usage for the approval (if present) and bridging transactions
    // and update gas limits for transactions
    let simulation_results = state
        .providers
        .simulation_provider
        .simulate_bundled_transactions(routes.clone(), HashMap::new(), state.metrics.clone())
        .await?;
    for (index, simulation_result) in simulation_results.simulation_results.iter().enumerate() {
        // Making sure the simulation input matches the transaction input
        let curr_route = routes.get_mut(index).ok_or_else(|| {
            RpcError::SimulationFailed("The route index is out of bounds".to_string())
        })?;
        if simulation_result.transaction.input != curr_route.input {
            return Err(RpcError::SimulationFailed(
                "The input for the simulation result does not match the input for the transaction"
                    .into(),
            ));
        }

        curr_route.gas_limit = U64::from(
            (simulation_result.transaction.gas * (100 + ESTIMATED_GAS_SLIPPAGE as u64)) / 100,
        );

        // Get the transaction type for metrics based on the assumption that the first transaction is an approval
        // and the rest are bridging transactions
        let tx_type = if simulation_results.simulation_results.len() == 1 {
            // If there is only one transaction, it's a bridging transaction
            ChainAbstractionTransactionType::Bridge
        } else if index == 0 {
            ChainAbstractionTransactionType::Approve
        } else {
            ChainAbstractionTransactionType::Bridge
        };
        state.metrics.add_ca_gas_estimation(
            simulation_result.transaction.gas,
            curr_route.chain_id.clone(),
            tx_type,
        );
    }

    // Save the bridging transaction to the IRN
    let orchestration_id = Uuid::new_v4().to_string();
    let bridging_status_item = StorageBridgingItem {
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        chain_id: initial_tx_chain_id.clone(),
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

    // Analytics
    {
        let origin = headers
            .get("origin")
            .map(|v| v.to_str().unwrap_or("invalid_header").to_string());
        let (country, continent, region) = state
            .analytics
            .lookup_geo_data(
                network::get_forwarded_ip(&headers).unwrap_or_else(|| connect_info.0.ip()),
            )
            .map(|geo| (geo.country, geo.continent, geo.region))
            .unwrap_or((None, None, None));
        state
            .analytics
            .chain_abstraction_funding(ChainAbstractionFundingInfo::new(
                query_params.project_id.to_string(),
                origin.clone(),
                region.clone(),
                country.clone(),
                continent.clone(),
                query_params.sdk_version.clone(),
                query_params.sdk_type.clone(),
                orchestration_id.clone(),
                bridge_chain_id.clone(),
                bridge_contract.to_string(),
                bridge_token_symbol.clone(),
                bridging_amount.to_string(),
            ));
        state
            .analytics
            .chain_abstraction_bridging(ChainAbstractionBridgingInfo::new(
                query_params.project_id.to_string(),
                origin.clone(),
                region.clone(),
                country.clone(),
                continent.clone(),
                query_params.sdk_version.clone(),
                query_params.sdk_type.clone(),
                orchestration_id.clone(),
                bridge_chain_id.clone(),
                bridge_contract.to_string(),
                bridge_token_symbol.clone(),
                initial_tx_chain_id.clone(),
                to_address.to_string(),
                initial_tx_token_symbol.clone(),
                bridging_amount.to_string(),
                final_bridging_fee.to_string(),
            ));
        state
            .analytics
            .chain_abstraction_initial_tx(ChainAbstractionInitialTxInfo::new(
                query_params.project_id.to_string(),
                origin,
                region,
                country,
                continent,
                query_params.sdk_version,
                query_params.sdk_type,
                orchestration_id.clone(),
                from_address.to_string(),
                to_address.to_string(),
                asset_transfer_value.to_string(),
                initial_tx_chain_id.clone(),
                to_address.to_string(),
                initial_tx_token_symbol.clone(),
            ));
    }

    state
        .metrics
        .add_ca_routes_found(construct_metrics_bridging_route(
            bridge_chain_id.clone(),
            bridge_contract.to_string(),
            request_payload.transaction.chain_id.clone(),
            asset_transfer_contract.to_string(),
        ));

    return Ok(
        Json(PrepareResponse::Success(PrepareResponseSuccess::Available(
            PrepareResponseAvailable {
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
        .into_response(),
    );
}

fn construct_metrics_bridging_route(
    from_chain_id: String,
    from_contract: String,
    to_chain_id: String,
    to_contract: String,
) -> String {
    format!(
        "{}:{}->{}:{}",
        from_chain_id, from_contract, to_chain_id, to_contract
    )
}
