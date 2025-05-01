use {
    crate::{
        error::RpcError,
        handlers::MessageSource,
        providers::{
            tenderly::{AssetChangeType, TokenStandard},
            SimulationProvider,
        },
        utils::{crypto::get_erc20_balance, token_amount::TokenAmount},
        Metrics,
    },
    alloy::primitives::{Address, Bytes, B256, U256},
    assets::{Eip155OrSolanaStatic, SimulationParams, BRIDGING_ASSETS},
    ethers::{types::H160 as EthersH160, utils::keccak256},
    futures_util::{stream::FuturesUnordered, StreamExt},
    serde::{Deserialize, Serialize},
    std::{cmp::Ordering, collections::HashMap, sync::Arc},
    tracing::debug,
    yttrium::chain_abstraction::{
        api::prepare::Eip155OrSolanaAddress,
        solana::{self, SolanaRpcClient},
    },
};

pub mod assets;
pub mod lifi;
pub mod route;
pub mod status;

/// How much to multiply the bridging fee amount to cover bridging fee volatility
pub const BRIDGING_FEE_SLIPPAGE: i16 = 250; // 250%

/// Bridging timeout in seconds
pub const BRIDGING_TIMEOUT: u64 = 1800; // 30 minutes

/// The status polling interval in ms for the client
pub const STATUS_POLLING_INTERVAL: u64 = 3000; // 3 seconds

/// Serialized bridging request item schema to store it in the IRN database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBridgingItem {
    created_at: u64,
    chain_id: String,
    wallet: Address,
    contract: Address,
    amount_current: U256,
    amount_expected: U256,
    status: BridgingStatus,
    error_reason: Option<String>,
}

/// Bridging status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BridgingStatus {
    Pending,
    Completed,
    Error,
}

/// Return available assets names and contract addresses for the given chain_id
pub fn get_bridging_assets_contracts_for_chain(
    chain_id: &str,
) -> Vec<(String, Eip155OrSolanaStatic)> {
    BRIDGING_ASSETS
        .entries()
        .filter_map(|(token_symbol, asset_entry)| {
            asset_entry
                .contracts
                .entries()
                .find(|(chain, _)| **chain == chain_id)
                .map(|(_, contract_address)| (token_symbol.to_string(), *contract_address))
        })
        .collect()
}

/// Returns simulation params for the bridging asset
pub fn get_simulation_params_for_asset(asset_name: &str) -> Option<&SimulationParams> {
    BRIDGING_ASSETS
        .entries()
        .find(|(name, _)| **name == asset_name)
        .map(|(_, asset_entry)| &asset_entry.simulation)
}

/// Check is the address is supported bridging asset and return the token symbol and decimals
pub fn find_supported_bridging_asset(
    chain_id: &str,
    contract: Eip155OrSolanaAddress,
) -> Option<(String, u8)> {
    for (symbol, asset_entry) in BRIDGING_ASSETS.entries() {
        for (chain, contract_address) in asset_entry.contracts.entries() {
            if *chain == chain_id && contract == contract_address.into_eip155_or_solana_address() {
                return Some((symbol.to_string(), asset_entry.metadata.decimals));
            }
        }
    }
    None
}

pub struct BridgingAsset {
    pub chain_id: String,
    pub account: Eip155OrSolanaAddress,
    pub token_symbol: String,
    pub contract_address: Eip155OrSolanaAddress,
    pub decimals: u8,
    pub current_balance: U256,
}

/// Checking available assets amount for bridging excluding the initial transaction
/// asset, prioritizing the asset with the highest balance or the asset with the
/// same symbol to avoid unnecessary swapping
#[allow(clippy::too_many_arguments)]
pub async fn check_bridging_for_erc20_transfer(
    rpc_project_id: String,
    session_id: Option<String>,
    value: U256,
    // List of CAIP-10 accounts to check for funds to bridge. Empty CAIP-2 field indicates any chain that matches the address type
    accounts: Vec<(Option<String>, Eip155OrSolanaAddress)>,
    // Exclude the initial transaction asset from the check
    exclude_chain_id: String,
    exclude_contract_address: Eip155OrSolanaAddress,
    // Using the same asset as a priority for bridging
    token_symbol_priority: String,
    // Applying token decimals for the value to compare between different tokens
    amount_token_decimals: u8,
    // Use a single asset with symbol
    only_token_symbol: Option<String>,
    // Exclude token symbol
    exclude_token_symbol: Option<String>,
    solana_rpc_client: Arc<SolanaRpcClient>,
) -> Result<Option<BridgingAsset>, RpcError> {
    // Check ERC20 tokens balance for each of supported assets

    let mut balances_to_check: Vec<(
        String,                // token symbol
        String,                // chain id
        u8,                    // decimals
        Eip155OrSolanaAddress, // contract address
        Eip155OrSolanaAddress, // account address
    )> = Vec::new();

    // Gather all the balances to check
    for (token_symbol, asset_entry) in BRIDGING_ASSETS.entries() {
        for (chain_id, contract_address) in asset_entry.contracts.entries() {
            // Exclude the initial transaction asset
            if *chain_id == exclude_chain_id
                && contract_address.into_eip155_or_solana_address() == exclude_contract_address
            {
                continue;
            }
            // Check for the single token symbol if provided
            if let Some(only_token_symbol) = only_token_symbol.clone() {
                if *token_symbol != only_token_symbol {
                    continue;
                }
            }
            // Exclude token symbol
            if let Some(exclude_token_symbol) = exclude_token_symbol.clone() {
                if *token_symbol == exclude_token_symbol {
                    continue;
                }
            }

            let accounts = accounts.iter().filter_map(|(cid, address)| {
                if address.same_kind_as(&contract_address.into_eip155_or_solana_address()) {
                    if let Some(cid) = cid {
                        if cid == chain_id {
                            Some(address.clone())
                        } else {
                            None
                        }
                    } else {
                        Some(address.clone())
                    }
                } else {
                    None
                }
            });

            for account in accounts {
                balances_to_check.push((
                    token_symbol.to_string(),
                    (*chain_id).to_string(),
                    asset_entry.metadata.decimals,
                    contract_address.into_eip155_or_solana_address(),
                    account,
                ));
            }
        }
    }

    let mut handles = FuturesUnordered::new();

    // Dispatch all of the balance checks
    for (token_symbol, chain_id, decimals, contract, account) in balances_to_check {
        assert!(contract.same_kind_as(&account));
        match chain_id {
            chain_id if chain_id.starts_with("eip155:") => {
                let rpc_project_id = rpc_project_id.clone();
                let session_id = session_id.clone();
                handles.push(futures_util::future::Either::Left(async move {
                    let balance = U256::from_be_bytes(
                        // TODO switch to alloy and use ProviderPool
                        get_erc20_balance(
                            &chain_id,
                            EthersH160::from(<[u8; 20]>::from(*contract.as_eip155().unwrap())),
                            EthersH160::from(<[u8; 20]>::from(*account.as_eip155().unwrap())),
                            &rpc_project_id,
                            MessageSource::ChainAgnosticCheck,
                            session_id,
                        )
                        .await?
                        .into(),
                    );
                    Result::<_, RpcError>::Ok((
                        token_symbol,
                        chain_id,
                        decimals,
                        contract,
                        account,
                        balance,
                    ))
                }));
            }
            chain_id if chain_id.starts_with("solana:") => {
                let solana_rpc_client = solana_rpc_client.clone();
                handles.push(futures_util::future::Either::Right(async move {
                    let balance = solana_rpc_client
                        .get_token_account_balance(&solana::get_associated_token_address(
                            account
                                .as_solana()
                                .expect("Account is not a solana address"),
                            contract
                                .as_solana()
                                .expect("Contract is not a solana address"),
                        ))
                        .await
                        .map_err(|e| {
                            RpcError::CryptoUitlsError(
                                crate::utils::crypto::CryptoUitlsError::ProviderError(format!(
                                    "Failed to get solana token account balance: {}",
                                    e
                                )),
                            )
                        })?
                        .amount
                        .parse::<U256>()
                        .map_err(|e| {
                            RpcError::CryptoUitlsError(
                                crate::utils::crypto::CryptoUitlsError::ProviderError(format!(
                                    "Failed to parse solana token account balance: {}",
                                    e
                                )),
                            )
                        })?;
                    Ok((token_symbol, chain_id, decimals, contract, account, balance))
                }));
            }
            _ => {
                unreachable!("Invalid chain id: {}", chain_id);
            }
        }
    }

    // Making the check for each chain_id and use the asset with the highest balance
    let mut bridging_asset_found: Option<BridgingAsset> = None;

    // Consume the results as they come in, possibly aborting the operation early if priority asset is found
    while let Some(result) = handles.next().await {
        let (token_symbol, chain_id, decimals, contract_address, account, current_balance) =
            result?;
        // Check if the balance compared to the transfer value is enough, applied to the transfer token decimals
        if TokenAmount::new(current_balance, decimals)
            >= TokenAmount::new(value, amount_token_decimals)
        {
            // Use the priority asset if found
            if token_symbol == token_symbol_priority {
                return Ok(Some(BridgingAsset {
                    chain_id: chain_id.clone(),
                    account,
                    token_symbol: token_symbol.clone(),
                    contract_address,
                    current_balance,
                    decimals,
                }));
            }

            // or use the asset with the highest found balance
            if let Some(BridgingAsset {
                current_balance: existing_balance,
                decimals: existing_decimals,
                ..
            }) = &bridging_asset_found
            {
                if TokenAmount::new(current_balance, decimals)
                    <= TokenAmount::new(*existing_balance, *existing_decimals)
                {
                    continue;
                }
            }
            bridging_asset_found = Some(BridgingAsset {
                chain_id: chain_id.clone(),
                account,
                token_symbol: token_symbol.clone(),
                contract_address,
                current_balance,
                decimals,
            });
        }
    }

    Ok(bridging_asset_found)
}

/// Compute the simulation state override balance for a given balance
pub fn compute_simulation_balance(balance: u128) -> B256 {
    let mut buf = [0u8; 32];
    buf[16..32].copy_from_slice(&balance.to_be_bytes());
    B256::from(buf)
}

/// Compute the storage slot for a given address and slot number to use in the
/// simulation state overrides
/// https://docs.tenderly.co/simulations/state-overrides#storage-slot-calculation
pub fn compute_simulation_storage_slot(address: Address, slot_number: u64) -> B256 {
    let mut input = [0u8; 64];
    // Place the address as the first 32-byte segment
    input[0..32].copy_from_slice(address.into_word().as_slice());
    // Place the u64 slot_number at the end of the second 32-byte segment
    input[56..64].copy_from_slice(&slot_number.to_be_bytes());
    B256::from(&keccak256(input))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Erc20AssetChange {
    pub chain_id: String,
    pub asset_contract: Address,
    pub amount: U256,
    pub receiver: Address,
}

/// Get the ERC20 assets changes and gas used from the transaction simulation result
pub async fn get_assets_changes_from_simulation(
    simulation_provider: Arc<dyn SimulationProvider>,
    chain_id: String,
    from: Address,
    to: Address,
    input: Bytes,
    metrics: Arc<Metrics>,
) -> Result<(Vec<Erc20AssetChange>, u64), RpcError> {
    // Fill the state overrides for the source address for each of the supported
    // assets on the initial tx chain for the balance slot
    let state_overrides = {
        let mut state_overrides = HashMap::new();
        let assets_contracts = get_bridging_assets_contracts_for_chain(&chain_id);
        let mut account_state = HashMap::new();
        for (asset_name, asset_contract) in assets_contracts {
            let asset_contract = if let Eip155OrSolanaStatic::Eip155(contract) = asset_contract {
                contract
            } else {
                continue;
            };
            let Some(simulation_params) = get_simulation_params_for_asset(&asset_name) else {
                continue;
            };
            let balance_storage_slot = *simulation_params
                .balance_storage_slots
                .get(&chain_id)
                .ok_or_else(|| {
                    RpcError::InvalidConfiguration(format!(
                        "Contract balance storage slot for simulation is not present for {} on {}",
                        asset_name, chain_id
                    ))
                })?;
            account_state.insert(
                compute_simulation_storage_slot(from, balance_storage_slot),
                compute_simulation_balance(simulation_params.balance),
            );
            state_overrides.insert(asset_contract, account_state.clone());
        }
        state_overrides
    };

    let simulation_result = &simulation_provider
        .simulate_transaction(chain_id.clone(), from, to, input, state_overrides, metrics)
        .await?;
    let gas_used = simulation_result.transaction.gas;

    if simulation_result
        .transaction
        .transaction_info
        .asset_changes
        .is_none()
    {
        debug!("The transaction does not change any assets");
        return Ok((vec![], gas_used));
    }

    let mut asset_changes = Vec::new();
    for asset_changed in simulation_result
        .transaction
        .transaction_info
        .asset_changes
        .clone()
        .unwrap_or_default()
    {
        if asset_changed.asset_type.clone() == AssetChangeType::Transfer
            && asset_changed.token_info.standard.clone() == TokenStandard::Erc20
            && asset_changed.to.is_some()
            && asset_changed.token_info.contract_address.is_some()
        {
            asset_changes.push(Erc20AssetChange {
                chain_id: chain_id.clone(),
                asset_contract: asset_changed
                    .token_info
                    .contract_address
                    .unwrap_or_default(),
                amount: asset_changed.raw_amount,
                receiver: asset_changed.to.unwrap_or_default(),
            })
        }
    }

    Ok((asset_changes, gas_used))
}

/// Convert the amount between different decimals
pub fn convert_amount(amount: U256, from_decimals: u8, to_decimals: u8) -> U256 {
    match from_decimals.cmp(&to_decimals) {
        Ordering::Equal => amount,
        Ordering::Greater => {
            // Reducing decimals: divide by 10^(from_decimals - to_decimals)
            let diff = from_decimals - to_decimals;
            let exp = U256::from(diff as u64);
            let factor = U256::from(10).pow(exp);
            // FIXME possible truncation error here? This `convert_amount()` function may be unsafe to use in some cases
            amount / factor
        }
        Ordering::Less => {
            // Increasing decimals: multiply by 10^(to_decimals - from_decimals)
            let diff = to_decimals - from_decimals;
            let exp = U256::from(diff as u64);
            let factor = U256::from(10).pow(exp);
            amount * factor
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_convert_amount() {
        let amount = U256::from_str("12345678901234567890").unwrap();
        let converted = convert_amount(amount, 18, 18);
        assert_eq!(converted, amount);

        // Converting 500 USDT (6 decimals) to 18 decimals.
        let usdt_amount = U256::from(500_000_000u64);
        let converted = convert_amount(usdt_amount, 6, 18);
        let expected = U256::from_str("500000000000000000000").unwrap();
        assert_eq!(converted, expected);

        // Converting 500 DAI (18 decimals) to 6 decimals.
        let dai_amount = U256::from_str("500000000000000000000").unwrap();
        let converted = convert_amount(dai_amount, 18, 6);
        let expected = U256::from(500_000_000u64);
        assert_eq!(converted, expected);
    }
}
