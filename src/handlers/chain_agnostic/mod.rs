use {
    crate::{
        error::RpcError,
        handlers::MessageSource,
        providers::{
            tenderly::{AssetChangeType, TokenStandard},
            SimulationProvider,
        },
        utils::crypto::get_erc20_contract_balance,
        Metrics,
    },
    alloy::primitives::{Address, B256, U256},
    assets::{SimulationParams, BRIDGING_ASSETS},
    ethers::{types::H160 as EthersH160, utils::keccak256},
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, str::FromStr, sync::Arc},
    tracing::debug,
    yttrium::chain_abstraction::api::Transaction,
};

pub mod assets;
pub mod route;
pub mod status;

/// How much to multiply the amount by when bridging to cover bridging differences
pub const BRIDGING_AMOUNT_SLIPPAGE: i8 = 50; // 50%

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
pub fn get_bridging_assets_contracts_for_chain(chain_id: &str) -> Vec<(String, Address)> {
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
pub fn find_supported_bridging_asset(chain_id: &str, contract: Address) -> Option<(String, u8)> {
    for (symbol, asset_entry) in BRIDGING_ASSETS.entries() {
        for (chain, contract_address) in asset_entry.contracts.entries() {
            if *chain == chain_id && contract == *contract_address {
                return Some((symbol.to_string(), asset_entry.metadata.decimals));
            }
        }
    }
    None
}

/// Checking ERC20 balances for given address for provided ERC20 contracts
pub async fn check_erc20_balances(
    project_id: String,
    address: Address,
    chain_id: String,
    erc2_contracts: Vec<Address>,
) -> Result<Vec<(Address, U256)>, RpcError> {
    let mut balances = Vec::new();
    // Check the ERC20 tokens balance for each of supported assets
    // TODO: Use the balance provider instead of looping
    for contract in erc2_contracts {
        let erc20_balance = get_erc20_contract_balance(
            &chain_id,
            EthersH160::from(<[u8; 20]>::from(contract)),
            EthersH160::from(<[u8; 20]>::from(address)),
            &project_id,
            MessageSource::ChainAgnosticCheck,
        )
        .await?;
        balances.push((contract, U256::from_be_bytes(erc20_balance.into())));
    }
    Ok(balances)
}

pub struct BridgingAsset {
    pub chain_id: String,
    pub token_symbol: String,
    pub contract_address: Address,
    pub decimals: u8,
    pub current_balance: U256,
}

/// Checking available assets amount for bridging excluding the initial transaction
/// asset, prioritizing the asset with the highest balance or the asset with the
/// same symbol to avoid unnecessary swapping
pub async fn check_bridging_for_erc20_transfer(
    rpc_project_id: String,
    value: U256,
    sender: Address,
    // Exclude the initial transaction asset from the check
    exclude_chain_id: String,
    exclude_contract_address: Address,
    // Using the same asset as a priority for bridging
    token_symbol_priority: String,
) -> Result<Option<BridgingAsset>, RpcError> {
    // Check ERC20 tokens balance for each of supported assets
    let mut contracts_per_chain: HashMap<(String, String, u8), Vec<String>> = HashMap::new();
    for (token_symbol, asset_entry) in BRIDGING_ASSETS.entries() {
        for (chain_id, contract_address) in asset_entry.contracts.entries() {
            if *chain_id == exclude_chain_id && *contract_address == exclude_contract_address {
                continue;
            }
            contracts_per_chain
                .entry((
                    token_symbol.to_string(),
                    (*chain_id).to_string(),
                    asset_entry.metadata.decimals,
                ))
                .or_default()
                .push((*contract_address).to_string());
        }
    }
    // Making the check for each chain_id and use the asset with the highest balance
    let mut bridging_asset_found = None;
    for ((token_symbol, chain_id, decimals), contracts) in contracts_per_chain {
        let erc20_balances = check_erc20_balances(
            rpc_project_id.clone(),
            sender,
            chain_id.clone(),
            contracts
                .iter()
                .map(|c| Address::from_str(c).unwrap_or_default())
                .collect(),
        )
        .await?;
        for (contract_address, current_balance) in erc20_balances {
            if current_balance >= value {
                // Use the priority asset if found
                if token_symbol == token_symbol_priority {
                    return Ok(Some(BridgingAsset {
                        chain_id: chain_id.clone(),
                        token_symbol: token_symbol.clone(),
                        contract_address,
                        current_balance,
                        decimals,
                    }));
                }

                // or use the asset with the highest found balance
                if let Some(BridgingAsset {
                    current_balance: existing_balance,
                    ..
                }) = &bridging_asset_found
                {
                    if current_balance <= *existing_balance {
                        continue;
                    }
                }
                bridging_asset_found = Some(BridgingAsset {
                    chain_id: chain_id.clone(),
                    token_symbol: token_symbol.clone(),
                    contract_address,
                    current_balance,
                    decimals,
                });
            }
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
    transaction: &Transaction,
    metrics: Arc<Metrics>,
) -> Result<(Vec<Erc20AssetChange>, u64), RpcError> {
    // Fill the state overrides for the source address for each of the supported
    // assets on the initial tx chain for the balance slot
    let state_overrides = {
        let mut state_overrides = HashMap::new();
        let assets_contracts =
            get_bridging_assets_contracts_for_chain(&transaction.chain_id.clone());
        let mut account_state = HashMap::new();
        for (asset_name, asset_contract) in assets_contracts {
            let Some(simulation_params) = get_simulation_params_for_asset(&asset_name) else {
                continue;
            };
            account_state.insert(
                compute_simulation_storage_slot(
                    transaction.from,
                    simulation_params.balance_storage_slot,
                ),
                compute_simulation_balance(simulation_params.balance),
            );
            state_overrides.insert(asset_contract, account_state.clone());
        }
        state_overrides
    };

    let simulation_result = &simulation_provider
        .simulate_transaction(
            transaction.chain_id.clone(),
            transaction.from,
            transaction.to,
            transaction.input.clone(),
            state_overrides,
            metrics,
        )
        .await?;
    let gas_used = simulation_result.transaction.gas_used;

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
        {
            asset_changes.push(Erc20AssetChange {
                chain_id: transaction.chain_id.clone(),
                asset_contract: asset_changed.token_info.contract_address,
                amount: asset_changed.raw_amount,
                receiver: asset_changed.to,
            })
        }
    }

    Ok((asset_changes, gas_used))
}
