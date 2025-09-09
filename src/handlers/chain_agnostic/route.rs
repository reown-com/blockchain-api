use {
    super::{
        assets::NATIVE_TOKEN_ADDRESS, check_bridging_for_erc20_transfer, convert_amount,
        find_supported_bridging_asset, get_assets_changes_from_simulation,
        nonce_manager::NonceManager, BridgingStatus, StorageBridgingItem, BRIDGING_FEE_SLIPPAGE,
        STATUS_POLLING_INTERVAL,
    },
    crate::{
        analytics::{
            ChainAbstractionBridgingInfo, ChainAbstractionFundingInfo,
            ChainAbstractionInitialTxInfo, MessageSource,
        },
        error::RpcError,
        handlers::{chain_agnostic::lifi::caip2_to_lifi_chain_id, self_provider, SdkInfoParams},
        metrics::{ChainAbstractionNoBridgingNeededType, ChainAbstractionTransactionType},
        state::AppState,
        storage::irn::OperationType,
        utils::{
            crypto::{
                convert_alloy_address_to_h160, decode_erc20_transfer_data, get_erc20_balance,
                get_gas_estimate, Erc20FunctionType,
            },
            network,
            simple_request_json::SimpleRequestJson,
        },
    },
    alloy::primitives::{Address, Bytes, U256, U64},
    axum::{
        extract::{ConnectInfo, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::{HeaderMap, StatusCode},
    serde::{Deserialize, Serialize},
    serde_json::json,
    std::{
        collections::HashMap,
        net::SocketAddr,
        str::FromStr,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    },
    tracing::{debug, error},
    uuid::Uuid,
    wc::metrics::{future_metrics, FutureExt},
    yttrium::{
        chain_abstraction::{
            api::{
                prepare::{
                    BridgingError, Eip155OrSolanaAddress, FundingMetadata,
                    InitialTransactionMetadata, Metadata, PrepareRequest, PrepareResponse,
                    PrepareResponseAvailable, PrepareResponseError, PrepareResponseNotRequired,
                    PrepareResponseSuccess, RouteQueryParams, SolanaTransaction, Transactions,
                },
                Transaction,
            },
            solana::{
                self, SolanaCommitmentConfig, SolanaParsePubkeyError, SolanaPubkey,
                SolanaRpcClient, SolanaVersionedTransaction,
            },
        },
        erc20::ERC20,
    },
};

// Slippage for the gas estimation
const ESTIMATED_GAS_SLIPPAGE: i16 = 500; // Temporarily x5 slippage to cover the volatility

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuoteRoute {
    pub to_amount: String,
}

pub async fn handler_v1(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query_params: Query<RouteQueryParams>,
    SimpleRequestJson(request_payload): SimpleRequestJson<PrepareRequest>,
) -> Result<Json<PrepareResponseV1>, RpcError> {
    handler_internal(state, connect_info, headers, query_params, request_payload)
        .with_metrics(future_metrics!("handler:ca_route"))
        .await
        .map(|Json(j)| Json(j.into()))
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrepareResponseV1 {
    Success(PrepareResponseSuccessV1),
    Error(PrepareResponseError),
}

impl From<PrepareResponse> for PrepareResponseV1 {
    fn from(value: PrepareResponse) -> Self {
        match value {
            PrepareResponse::Success(success) => PrepareResponseV1::Success(success.into()),
            PrepareResponse::Error(error) => PrepareResponseV1::Error(error),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrepareResponseSuccessV1 {
    Available(PrepareResponseAvailableV1),
    NotRequired(PrepareResponseNotRequired),
}

impl From<PrepareResponseSuccess> for PrepareResponseSuccessV1 {
    fn from(value: PrepareResponseSuccess) -> Self {
        match value {
            PrepareResponseSuccess::Available(available) => {
                PrepareResponseSuccessV1::Available(available.into())
            }
            PrepareResponseSuccess::NotRequired(not_required) => {
                PrepareResponseSuccessV1::NotRequired(not_required)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareResponseAvailableV1 {
    pub orchestration_id: String,
    pub initial_transaction: Transaction,
    pub transactions: Vec<Transaction>,
    pub metadata: MetadataV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataV1 {
    pub funding_from: Vec<FundingMetadataV1>,
    pub initial_transaction: InitialTransactionMetadata,
    pub check_in: u64,
}

impl From<Metadata> for MetadataV1 {
    fn from(value: Metadata) -> Self {
        Self {
            funding_from: value.funding_from.into_iter().map(|f| f.into()).collect(),
            initial_transaction: value.initial_transaction,
            check_in: value.check_in,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingMetadataV1 {
    pub chain_id: String,
    pub token_contract: String,
    pub symbol: String,
    pub amount: U256,
    pub bridging_fee: U256,
    pub decimals: u8,
}

impl From<FundingMetadata> for FundingMetadataV1 {
    fn from(value: FundingMetadata) -> Self {
        Self {
            chain_id: value.chain_id,
            token_contract: value
                .token_contract
                .as_eip155()
                .expect("V1 response only supports EIP155 funding token contracts")
                .to_string()
                .to_ascii_lowercase(),
            symbol: value.symbol,
            amount: value.amount,
            bridging_fee: value.bridging_fee,
            decimals: value.decimals,
        }
    }
}

impl From<PrepareResponseAvailable> for PrepareResponseAvailableV1 {
    fn from(value: PrepareResponseAvailable) -> Self {
        PrepareResponseAvailableV1 {
            orchestration_id: value.orchestration_id,
            initial_transaction: value.initial_transaction,
            transactions: value
                .transactions
                .into_iter()
                // NOTE: this is a temporary solution to support the legacy transactions format. However it will break in the future when multiple "routes" are returned
                .flat_map(|t| match t {
                    Transactions::Eip155(transactions) => transactions,
                    Transactions::Solana(_transactions) => {
                        error!("Solana txns not supported in v1 format");
                        Vec::new()
                    }
                })
                .collect(),
            metadata: value.metadata.into(),
        }
    }
}

fn no_bridging_needed_response(initial_transaction: Transaction) -> Json<PrepareResponse> {
    Json(PrepareResponse::Success(
        PrepareResponseSuccess::NotRequired(PrepareResponseNotRequired {
            initial_transaction,
            transactions: vec![],
        }),
    ))
}

pub async fn handler_v2(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query_params: Query<RouteQueryParams>,
    SimpleRequestJson(request_payload): SimpleRequestJson<PrepareRequest>,
) -> Result<Json<PrepareResponse>, RpcError> {
    handler_internal(state, connect_info, headers, query_params, request_payload)
        .with_metrics(future_metrics!("ca_route"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query_params): Query<RouteQueryParams>,
    request_payload: PrepareRequest,
) -> Result<Json<PrepareResponse>, RpcError> {
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

    // Check if the transaction value is non zero it's a native token transfer
    let (
        is_initial_tx_native_token_transfer,
        asset_transfer_value,
        asset_transfer_contract,
        asset_transfer_receiver,
        initial_tx_gas_used,
    ) = if first_call.value > U256::ZERO {
        let is_initial_tx_native_token_transfer = true;
        let asset_transfer_value = first_call.value;
        let asset_transfer_contract = NATIVE_TOKEN_ADDRESS;
        let asset_transfer_receiver = first_call.to;
        let simulation_result = get_assets_changes_from_simulation(
            state.providers.simulation_provider.clone(),
            request_payload.transaction.chain_id.clone(),
            request_payload.transaction.from,
            first_call.to,
            first_call.input.clone(),
            state.metrics.clone(),
        )
        .await;
        let simulated_gas_used = match simulation_result {
            Ok(simulation_result) => simulation_result.1,
            Err(e) => {
                return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                    error: BridgingError::TransactionSimulationFailed,
                    reason: format!(
                        "The initial transaction (native token transfer) simulation failed due to an error: {e}"
                    ),
                })));
            }
        };

        let gas_used = simulated_gas_used;
        (
            is_initial_tx_native_token_transfer,
            asset_transfer_value,
            asset_transfer_contract,
            asset_transfer_receiver,
            gas_used,
        )
    } else {
        let is_initial_tx_native_token_transfer = false;
        // Decode the ERC20 transfer function data or use the simulation
        // to get the transfer asset and amount
        let (asset_transfer_contract, asset_transfer_value, asset_transfer_receiver, gas_used) =
            match decode_erc20_transfer_data(&first_call.input) {
                Ok((receiver, erc20_transfer_value)) => {
                    debug!(
                        "The transaction is an ERC20 transfer with value: {:?}",
                        erc20_transfer_value
                    );

                    // Check if the destination address is supported ERC20 asset contract
                    // return an error if not, since the simulation for the gas estimation
                    // will fail
                    if find_supported_bridging_asset(
                        &request_payload.transaction.chain_id.clone(),
                        Eip155OrSolanaAddress::Eip155(first_call.to),
                    )
                    .is_none()
                    {
                        error!(
                            "The destination address is not a supported bridging asset contract {}:{}",
                            request_payload.transaction.chain_id.clone(),
                            first_call.to
                        );
                        state.metrics.add_ca_no_bridging_needed(
                            ChainAbstractionNoBridgingNeededType::AssetNotSupported,
                        );
                        return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                            error: BridgingError::AssetNotSupported,
                            reason: format!(
                                "The initial transaction asset {}:{} is not supported for the bridging",
                                request_payload.transaction.chain_id.clone(),
                                first_call.to
                            ),
                        })));
                    };

                    // Get the ERC20 transfer gas estimation for the token contract
                    // and chain_id, or simulate the transaction to get the gas used
                    let gas_used = match state
                        .providers
                        .simulation_provider
                        .get_cached_gas_estimation(
                            &request_payload.transaction.chain_id.clone(),
                            first_call.to,
                            Some(Erc20FunctionType::Transfer),
                        )
                        .await?
                    {
                        Some(gas) => gas,
                        None => {
                            let simulation_result = get_assets_changes_from_simulation(
                                state.providers.simulation_provider.clone(),
                                request_payload.transaction.chain_id.clone(),
                                request_payload.transaction.from,
                                first_call.to,
                                first_call.input.clone(),
                                state.metrics.clone(),
                            )
                            .await;
                            let simulated_gas_used = match simulation_result {
                                Ok(simulation_result) => simulation_result.1,
                                Err(e) => {
                                    return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                                        error: BridgingError::TransactionSimulationFailed,
                                        reason: format!(
                                            "The initial transaction simulation failed due to an error: {e}"
                                        ),
                                    })));
                                }
                            };
                            state.metrics.add_ca_gas_estimation(
                                simulated_gas_used,
                                request_payload.transaction.chain_id.clone(),
                                ChainAbstractionTransactionType::Transfer,
                            );
                            // Save the initial tx gas estimation to the cache
                            {
                                let state = state.clone();
                                let initial_chain_id = request_payload.transaction.chain_id.clone();
                                tokio::spawn(async move {
                                    state
                                        .providers
                                        .simulation_provider
                                        .set_cached_gas_estimation(
                                            &initial_chain_id,
                                            first_call.to,
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

                    (first_call.to, erc20_transfer_value, receiver, gas_used)
                }
                _ => {
                    debug!(
                        "The transaction data is not an ERC20 transfer function, making a simulation"
                    );

                    let simulation_result = get_assets_changes_from_simulation(
                        state.providers.simulation_provider.clone(),
                        request_payload.transaction.chain_id.clone(),
                        request_payload.transaction.from,
                        first_call.to,
                        first_call.input.clone(),
                        state.metrics.clone(),
                    )
                    .await;

                    let (simulation_assets_changes, gas_used) = match simulation_result {
                        Ok(changes) => changes,
                        Err(e) => {
                            return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                                error: BridgingError::TransactionSimulationFailed,
                                reason: format!(
                                    "The initial transaction simulation failed due to an error: {e}"
                                ),
                            })));
                        }
                    };

                    let mut asset_transfer_value = U256::ZERO;
                    let mut asset_transfer_contract = Address::default();
                    let mut asset_transfer_receiver = Address::default();
                    for asset_change in simulation_assets_changes {
                        if find_supported_bridging_asset(
                            &asset_change.chain_id.clone(),
                            Eip155OrSolanaAddress::Eip155(asset_change.asset_contract),
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
                        return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                            error: BridgingError::AssetNotSupported,
                            reason: "The transaction does not change any supported bridging assets"
                                .to_string(),
                        })));
                    }

                    (
                        asset_transfer_contract,
                        asset_transfer_value,
                        asset_transfer_receiver,
                        gas_used,
                    )
                }
            };

        (
            is_initial_tx_native_token_transfer,
            asset_transfer_value,
            asset_transfer_contract,
            asset_transfer_receiver,
            gas_used,
        )
    };

    // Estimated gas multiplied by the slippage
    let initial_tx_gas_limit =
        U64::from((initial_tx_gas_used * (100 + ESTIMATED_GAS_SLIPPAGE as u64)) / 100);

    let mut nonce_manager = NonceManager::new(provider_pool.clone());
    nonce_manager.initialize_nonce(
        request_payload.transaction.chain_id.clone(),
        request_payload.transaction.from,
    );

    // Get the current balance of the ERC20 or native token and check if it's enough for the transfer
    // without bridging or calculate the top-up value
    let erc20_balance = get_erc20_balance(
        &request_payload.transaction.chain_id.clone(),
        convert_alloy_address_to_h160(asset_transfer_contract),
        convert_alloy_address_to_h160(request_payload.transaction.from),
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
        return Ok(no_bridging_needed_response(Transaction {
            from: request_payload.transaction.from,
            to: first_call.to,
            value: first_call.value,
            input: first_call.input.clone(),
            gas_limit: initial_tx_gas_limit,
            nonce: nonce_manager
                .get_nonce(
                    request_payload.transaction.chain_id.clone(),
                    request_payload.transaction.from,
                )
                .await??,
            chain_id: request_payload.transaction.chain_id.clone(),
        }));
    }
    let mut erc20_topup_value = asset_transfer_value - erc20_balance;

    // Check if the destination address is supported ERC20 asset contract
    // Attempt to destructure the result into symbol and decimals using a match expression
    let (initial_tx_token_symbol, initial_tx_token_decimals) = match find_supported_bridging_asset(
        &request_payload.transaction.chain_id,
        Eip155OrSolanaAddress::Eip155(asset_transfer_contract),
    ) {
        Some((symbol, decimals)) => (symbol, decimals),
        None => {
            error!("The changed asset is not a supported for the bridging");
            state
                .metrics
                .add_ca_no_bridging_needed(ChainAbstractionNoBridgingNeededType::AssetNotSupported);
            return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                error: BridgingError::AssetNotSupported,
                reason: format!(
                    "The initial transaction asset {}:{} is not supported for the bridging",
                    request_payload.transaction.chain_id, asset_transfer_contract
                ),
            })));
        }
    };

    let sol_rpc = "https://api.mainnet-beta.solana.com";
    let solana_rpc_client = Arc::new(SolanaRpcClient::new_with_commitment(
        sol_rpc.to_string(),
        SolanaCommitmentConfig::confirmed(), // TODO what commitment level should we use?
    ));

    // Check for possible bridging funds by iterating over supported assets
    // or return an insufficient funds error
    let Some(bridging_asset) = check_bridging_for_erc20_transfer(
        query_params.project_id.to_string(),
        query_params.session_id.clone(),
        erc20_topup_value,
        {
            let results = request_payload
                .accounts.iter()
                .map(|a| {
                    let mut caip10_parts = a.splitn(3, ":");
                    let namespace = caip10_parts.next().ok_or(RpcError::InvalidParameter(
                        "The account is not a valid CAIP-10 address: missing namespace".to_string(),
                    ))?;
                    let reference = caip10_parts.next().ok_or(RpcError::InvalidParameter(
                        "The account is not a valid CAIP-10 address: missing reference".to_string(),
                    ))?;
                    let address = caip10_parts.next().ok_or(RpcError::InvalidParameter(
                        "The account is not a valid CAIP-10 address: missing address".to_string(),
                    ))?;
                    Ok((namespace, reference, address))
                })
                .collect::<Result<Vec<_>, RpcError>>()?;
            let mut accounts = Vec::with_capacity(results.len());
            for result in results {
                let (namespace, reference, address) = result;
                accounts.push((
                    Some(format!("{namespace}:{reference}")),
                    match namespace {
                        "eip155" => Eip155OrSolanaAddress::Eip155(
                            Address::from_str(address).map_err(|_| {
                                RpcError::InvalidParameter(
                                    "The account is not a valid CAIP-10 address: invalid eip155 address".to_string(),
                                )
                            })?,
                        ),
                        "solana" => Eip155OrSolanaAddress::Solana(
                            SolanaPubkey::from_str(address).map_err(|_| {
                                RpcError::InvalidParameter(
                                    "The account is not a valid CAIP-10 address: invalid solana address".to_string(),
                                )
                            })?,
                        ),
                        namespace => {
                            return Err(RpcError::InvalidParameter(format!(
                                "The account is not a valid CAIP-10 address: invalid namespace: {namespace}",
                            )));
                        }
                    },
                ));
            }
            accounts.push((None, Eip155OrSolanaAddress::Eip155(request_payload.transaction.from)));
            accounts
        },
        request_payload.transaction.chain_id.clone(),
        Eip155OrSolanaAddress::Eip155(asset_transfer_contract),
        initial_tx_token_symbol.clone(),
        initial_tx_token_decimals,
        if is_initial_tx_native_token_transfer {
            Some("ETH".to_string())
        } else {
            None
        },
        if !is_initial_tx_native_token_transfer {
            Some("ETH".to_string())
        } else {
            None
        },
        solana_rpc_client.clone(),
    )
    .await?
    else {
        state.metrics.add_ca_insufficient_funds();
        return Ok(Json(PrepareResponse::Error(PrepareResponseError {
            error: BridgingError::InsufficientFunds,
            reason: format!(
                "No supported assets with at least {} amount were found in the address {}",
                erc20_topup_value, request_payload.transaction.from
            ),
        })));
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

    // Getting the current nonce for the address for the bridging transaction
    if bridge_chain_id.starts_with("eip155:") {
        nonce_manager.initialize_nonce(bridge_chain_id.clone(), request_payload.transaction.from);
    }

    let (routes, bridged_amount, final_bridging_fee) = match bridge_contract.clone() {
        Eip155OrSolanaAddress::Eip155(bridge_contract) if !query_params.use_lifi => {
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
                    request_payload.transaction.from,
                    state.metrics.clone(),
                )
                .await?;
            let Some(best_route) = quotes.first() else {
                state
                    .metrics
                    .add_ca_no_routes_found(construct_metrics_bridging_route(
                        bridge_chain_id.clone(),
                        bridge_contract.to_string(),
                        request_payload.transaction.chain_id.clone(),
                        asset_transfer_contract.to_string(),
                    ));
                return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                    error: BridgingError::NoRoutesAvailable,
                    reason: format!(
                        "No routes were found from {}:{} to {}:{} for an initial amount {}",
                        bridge_chain_id,
                        bridge_contract,
                        request_payload.transaction.chain_id,
                        asset_transfer_contract,
                        erc20_topup_value
                    ),
                })));
            };

            // Calculate the bridging fee based on the amount given from quotes
            let bridged_amount =
                serde_json::from_value::<QuoteRoute>(best_route.clone())?.to_amount;
            let bridged_amount = U256::from_str(&bridged_amount)
                .map_err(|_| RpcError::InvalidValue(bridged_amount))?;
            let bridged_amount =
                convert_amount(bridged_amount, initial_tx_token_decimals, bridge_decimals);

            // Handle negatie bridging fee on USDs swaps considering it as 0 fee
            // or calculate the bridging fee
            let bridging_fee = if bridged_amount > erc20_topup_value {
                error!(
                    "The bridged amount {} is higher than the requested amount {}",
                    bridged_amount, erc20_topup_value
                );
                U256::ZERO
            } else {
                erc20_topup_value - bridged_amount
            };

            // Calculate the required bridging topup amount with the bridging fee
            // and bridging fee * slippage to cover volatility
            let required_topup_amount = erc20_topup_value + bridging_fee;
            let required_topup_amount = ((bridging_fee * U256::from(BRIDGING_FEE_SLIPPAGE))
                / U256::from(100))
                + required_topup_amount;
            if current_bridging_asset_balance < required_topup_amount {
                let error_reason = format!(
                    "The current bridging asset balance on {} is {} less than the required topup amount:{}",
                    request_payload.transaction.from, current_bridging_asset_balance, required_topup_amount
                );
                error!(error_reason);
                state.metrics.add_ca_insufficient_funds();
                return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                    error: BridgingError::InsufficientFunds,
                    reason: error_reason,
                })));
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
                    request_payload.transaction.from,
                    state.metrics.clone(),
                )
                .await?;
            let Some(best_route) = quotes.first() else {
                state
                    .metrics
                    .add_ca_no_routes_found(construct_metrics_bridging_route(
                        bridge_chain_id.clone(),
                        bridge_contract.to_string(),
                        request_payload.transaction.chain_id.clone(),
                        asset_transfer_contract.to_string(),
                    ));
                return Ok(Json(PrepareResponse::Error(PrepareResponseError {
                    error: BridgingError::NoRoutesAvailable,
                    reason: format!(
                        "No routes were found from {}:{} to {}:{} for updated (fee included) amount: {}",
                        bridge_chain_id,
                        bridge_contract,
                        request_payload.transaction.chain_id,
                        asset_transfer_contract,
                        required_topup_amount
                    ),
                })));
            };

            // Check the final bridging amount from the quote
            let bridged_amount =
                serde_json::from_value::<QuoteRoute>(best_route.clone())?.to_amount;
            let bridged_amount = U256::from_str(&bridged_amount)
                .map_err(|_| RpcError::InvalidValue(bridged_amount))?;
            let bridged_amount =
                convert_amount(bridged_amount, initial_tx_token_decimals, bridge_decimals);

            if erc20_topup_value > bridged_amount {
                error!(
                    "The final bridged amount:{} is less than the topup amount:{}",
                    bridged_amount, erc20_topup_value
                );
                return Err(RpcError::BridgingFinalAmountLess);
            }

            let final_bridging_fee = bridged_amount - erc20_topup_value;

            // Build bridging transaction
            let bridge_tx = state
                .providers
                .chain_orchestrator_provider
                .build_bridging_tx(best_route.clone(), state.metrics.clone())
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
                        nonce: nonce_manager
                            .get_nonce(bridge_chain_id.clone(), request_payload.transaction.from)
                            .await??,
                        chain_id: format!("eip155:{}", bridge_tx.chain_id),
                    };
                    routes.push(approval_transaction);
                }
            }

            let mut bridging_transaction = Transaction {
                from: request_payload.transaction.from,
                to: bridge_tx.tx_target,
                value: bridge_tx.value,
                gas_limit: U64::ZERO,
                input: bridge_tx.tx_data,
                nonce: nonce_manager
                    .get_nonce(bridge_chain_id.clone(), request_payload.transaction.from)
                    .await??,
                chain_id: format!("eip155:{}", bridge_tx.chain_id),
            };

            // If the bridging transaction value is non zero, it's a native token transfer
            // and we can get the gas estimation by calling `eth_estimateGas` RPC method
            // instead of the simulation
            if !bridging_transaction.value.is_zero() {
                let gas_estimation = get_gas_estimate(
                    &bridging_transaction.chain_id.clone(),
                    bridging_transaction.from,
                    bridging_transaction.to,
                    bridging_transaction.value,
                    bridging_transaction.input.clone(),
                    &provider_pool.get_provider(
                        bridging_transaction.chain_id.clone(),
                        MessageSource::ChainAgnosticCheck,
                    ),
                )
                .await?;
                bridging_transaction.gas_limit =
                    U64::from((gas_estimation * (100 + ESTIMATED_GAS_SLIPPAGE as u64)) / 100);
            };
            routes.push(bridging_transaction.clone());

            // Estimate the gas usage for the approval (if present) and bridging transactions
            // and update gas limits for transactions
            // Skip the simulation if the bridging transaction is a native token transfer
            if routes.len() != 1 || bridging_transaction.gas_limit.is_zero() {
                let simulation_results = state
                    .providers
                    .simulation_provider
                    .simulate_bundled_transactions(
                        routes.clone(),
                        HashMap::new(),
                        state.metrics.clone(),
                    )
                    .await?;
                for (index, simulation_result) in
                    simulation_results.simulation_results.iter().enumerate()
                {
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
                        (simulation_result.transaction.gas * (100 + ESTIMATED_GAS_SLIPPAGE as u64))
                            / 100,
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
            }

            (
                vec![Transactions::Eip155(routes)],
                bridged_amount,
                final_bridging_fee,
            )
        }
        bridge_contract => {
            let quote = reqwest::Client::new()
                .get("https://li.quest/v1/quote/toAmount")
                .query(&json!({
                    "fromChain": caip2_to_lifi_chain_id(bridge_chain_id.as_str())?,
                    "toChain": caip2_to_lifi_chain_id(request_payload.transaction.chain_id.as_str())?,
                    "fromToken": bridge_contract.to_string(),
                    "toToken": asset_transfer_contract.to_string(),
                    "toAmount": erc20_topup_value.to_string(),
                    "fromAddress": bridging_asset.account.to_string(),
                    "toAddress": request_payload.transaction.from.to_string(),
                }))
                .send()
                .await
                .map_err(|e| RouteSolanaError::Internal(RouteSolanaInternalError::Request(e)))?
                .json::<serde_json::Value>()
                .await
                .map_err(|e| {
                    RouteSolanaError::Internal(RouteSolanaInternalError::JsonDeserialization(e))
                })?;
            debug!("Quote: {}", serde_json::to_string_pretty(&quote).unwrap());

            if bridge_chain_id.starts_with("solana:") {
                assert!(bridge_contract.as_solana().is_some());

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Quote {
                    action: Action,
                    transaction_request: TransactionRequest,
                }

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Action {
                    from_amount: U256,
                }

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct TransactionRequest {
                    data: String,
                }

                let quote = serde_json::from_value::<Quote>(quote).map_err(|e| {
                    RouteSolanaError::Internal(
                        RouteSolanaInternalError::LiFiQuoteDeserializationSolana(e),
                    )
                })?;
                debug!(
                    "Parsed quote: {}",
                    serde_json::to_string_pretty(&quote).unwrap()
                );

                let data = data_encoding::BASE64
                    .decode(quote.transaction_request.data.as_bytes())
                    .map_err(|e| {
                        RouteSolanaError::Internal(
                            RouteSolanaInternalError::TransactionRequestDecode(e),
                        )
                    })?;
                let tx = solana::bincode::deserialize::<SolanaVersionedTransaction>(&data)
                    .map_err(|e| {
                        RouteSolanaError::Internal(
                            RouteSolanaInternalError::TransactionRequestDeserialization(e),
                        )
                    })?;

                (
                    vec![Transactions::Solana(vec![SolanaTransaction {
                        from: *bridging_asset.account.as_solana().unwrap(),
                        chain_id: bridge_chain_id.clone(),
                        transaction: tx,
                    }])],
                    quote.action.from_amount,
                    quote.action.from_amount - erc20_topup_value,
                )
            } else if bridge_chain_id.starts_with("eip155:") {
                let bridge_contract = bridge_contract
                    .as_eip155()
                    .expect("Internal bug: the bridge contract should be an EIP-155 address when bridge_chain_id starts with eip155:");

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Quote {
                    action: Action,
                    estimate: Estimate,
                    transaction_request: TransactionRequest,
                }

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Action {
                    from_amount: U256,
                    from_token: FromToken,
                }

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct FromToken {
                    address: Address,
                }

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Estimate {
                    approval_address: Address,
                }

                #[derive(Debug, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct TransactionRequest {
                    chain_id: u64,
                    data: Bytes,
                    from: Address,
                    to: Address,
                    value: U256,
                    gas_limit: U256,
                    gas_price: U256, // should be unused, as this is legacy gas API
                }

                let quote = serde_json::from_value::<Quote>(quote).map_err(|e| {
                    RouteSolanaError::Internal(
                        RouteSolanaInternalError::LiFiQuoteDeserializationEip155(e),
                    )
                })?;
                debug!(
                    "Parsed quote: {}",
                    serde_json::to_string_pretty(&quote).unwrap()
                );

                let chain_id = format!("eip155:{}", quote.transaction_request.chain_id);
                assert_eq!(chain_id, bridge_chain_id);
                let from = quote.transaction_request.from;
                assert_eq!(from, request_payload.transaction.from);

                let mut txns = Vec::with_capacity(2);

                // Generate approval txn, if necessary
                {
                    // Approve 4x the necessary amount to avoid re-approving too often
                    const APPROVE_MULTIPLIER: u64 = 4;

                    // https://docs.li.fi/li.fi-api/li.fi-api/transferring-tokens-example#checking-and-setting-the-allowance
                    assert_eq!(bridge_contract, &quote.action.from_token.address);
                    assert_ne!(bridge_contract, &quote.estimate.approval_address);
                    let source_token = ERC20::new(
                        quote.action.from_token.address,
                        provider_pool.get_provider(
                            bridge_chain_id.clone(),
                            MessageSource::ChainAgnosticCheck,
                        ),
                    );
                    let allowance = source_token
                        .allowance(
                            request_payload.transaction.from,
                            quote.estimate.approval_address,
                        )
                        .call()
                        .await
                        .map_err(|e| {
                            RouteSolanaError::Internal(RouteSolanaInternalError::AllowanceCall(e))
                        })?
                        .remaining;
                    if allowance < quote.action.from_amount {
                        let approve_amount =
                            quote.action.from_amount * U256::from(APPROVE_MULTIPLIER);
                        let approve_tx =
                            source_token.approve(quote.estimate.approval_address, approve_amount);
                        txns.push(Transaction {
                            chain_id: chain_id.clone(),
                            from,
                            to: quote.action.from_token.address,
                            value: U256::ZERO,
                            input: approve_tx.calldata().clone(),
                            nonce: nonce_manager.get_nonce(chain_id.clone(), from).await??,
                            gas_limit: U64::from(100000), // TODO estimate gas
                        });
                    }
                }

                {
                    txns.push(Transaction {
                        chain_id: chain_id.clone(),
                        from,
                        to: quote.transaction_request.to,
                        value: quote.transaction_request.value,
                        input: quote.transaction_request.data,
                        nonce: nonce_manager.get_nonce(chain_id.clone(), from).await??,
                        gas_limit: U64::from(quote.transaction_request.gas_limit),
                    });
                }

                (
                    vec![Transactions::Eip155(txns)],
                    quote.action.from_amount,
                    quote.action.from_amount - erc20_topup_value,
                )
            } else {
                // Bug: This means that we have a supported asset on a non-supported chain
                unimplemented!("Unsupported chain ID: {}", bridge_chain_id)
            }
        }
    };

    // Save the bridging transaction to the IRN
    let orchestration_id = Uuid::new_v4().to_string();
    let bridging_status_item = StorageBridgingItem {
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        chain_id: request_payload.transaction.chain_id.clone(),
        wallet: request_payload.transaction.from,
        contract: asset_transfer_contract,
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
                bridged_amount.to_string(),
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
                request_payload.transaction.chain_id.clone(),
                first_call.to.to_string(),
                initial_tx_token_symbol.clone(),
                bridged_amount.to_string(),
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
                request_payload.transaction.from.to_string(),
                first_call.to.to_string(),
                asset_transfer_value.to_string(),
                request_payload.transaction.chain_id.clone(),
                first_call.to.to_string(),
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

    return Ok(Json(PrepareResponse::Success(
        PrepareResponseSuccess::Available(PrepareResponseAvailable {
            orchestration_id,
            initial_transaction: Transaction {
                from: request_payload.transaction.from,
                to: first_call.to,
                value: first_call.value,
                input: first_call.input.clone(),
                gas_limit: initial_tx_gas_limit,
                nonce: nonce_manager
                    .get_nonce(
                        request_payload.transaction.chain_id.clone(),
                        request_payload.transaction.from,
                    )
                    .await??,
                chain_id: request_payload.transaction.chain_id.clone(),
            },
            transactions: routes,
            metadata: Metadata {
                funding_from: vec![FundingMetadata {
                    chain_id: bridge_chain_id,
                    token_contract: bridge_contract,
                    symbol: bridge_token_symbol,
                    amount: bridged_amount,
                    bridging_fee: final_bridging_fee,
                    decimals: bridge_decimals,
                }],
                check_in: STATUS_POLLING_INTERVAL,
                initial_transaction: InitialTransactionMetadata {
                    transfer_to: asset_transfer_receiver,
                    amount: asset_transfer_value,
                    token_contract: asset_transfer_contract,
                    symbol: initial_tx_token_symbol,
                    decimals: initial_tx_token_decimals,
                },
            },
        }),
    )));
}

fn construct_metrics_bridging_route(
    from_chain_id: String,
    from_contract: String,
    to_chain_id: String,
    to_contract: String,
) -> String {
    format!("{from_chain_id}:{from_contract}->{to_chain_id}:{to_contract}")
}

#[derive(Debug, thiserror::Error)]
pub enum RouteSolanaError {
    #[error("Internal: {0}")]
    Internal(RouteSolanaInternalError),

    #[error("Request: {0}")]
    Request(RouteSolanaRequestError),
}

#[derive(Debug, thiserror::Error)]
pub enum RouteSolanaInternalError {
    #[error("Request: {0}")]
    Request(reqwest::Error),

    #[error("JSON deserialization: {0}")]
    JsonDeserialization(reqwest::Error),

    #[error("LiFi quote (Solana) deserialization: {0}")]
    LiFiQuoteDeserializationSolana(serde_json::Error),

    #[error("LiFi quote (EIP155) deserialization: {0}")]
    LiFiQuoteDeserializationEip155(serde_json::Error),

    #[error("Transaction request decode: {0}")]
    TransactionRequestDecode(data_encoding::DecodeError),

    #[error("Transaction request deserialization: {0}")]
    TransactionRequestDeserialization(solana::bincode::Error),

    #[error("Allowance call: {0}")]
    AllowanceCall(alloy::contract::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum RouteSolanaRequestError {
    #[error("Malformed Solana account: {0}")]
    MalformedSolanaAccount(SolanaParsePubkeyError),
}

impl RouteSolanaError {
    pub fn into_response(&self) -> Response {
        match self {
            Self::Internal(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::Request(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        }
    }
}
