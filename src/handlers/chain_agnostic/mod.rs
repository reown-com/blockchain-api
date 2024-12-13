use {
    crate::{
        error::RpcError,
        handlers::MessageSource,
        providers::{
            tenderly::{AssetChangeType, TokenStandard},
            SimulationProvider,
        },
        utils::crypto::get_erc20_contract_balance,
    },
    alloy::primitives::{address, Address, B256, U256},
    ethers::{types::H160 as EthersH160, utils::keccak256},
    phf::phf_map,
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, str::FromStr, sync::Arc},
    tracing::debug,
    yttrium::chain_abstraction::api::Transaction,
};

pub mod route;
pub mod status;

/// How much to multiply the amount by when bridging to cover bridging differences
pub const BRIDGING_AMOUNT_SLIPPAGE: i8 = 2; // 2%

/// Bridging timeout in seconds
pub const BRIDGING_TIMEOUT: u64 = 1800; // 30 minutes

/// Available assets for Bridging
pub static BRIDGING_AVAILABLE_ASSETS: phf::Map<&'static str, phf::Map<&'static str, Address>> = phf_map! {
  "USDC" => phf_map! {
      // Optimism
      "eip155:10" => address!("0b2c639c533813f4aa9d7837caf62653d097ff85"),
      // Base
      "eip155:8453" => address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
      // Arbitrum
      "eip155:42161" => address!("af88d065e77c8cC2239327C5EDb3A432268e5831"),
  },
};
pub const USDC_DECIMALS: u8 = 6;

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

/// Return available assets contracts addresses for the given chain_id
pub fn get_bridging_assets_contracts_for_chain(chain_id: &str) -> Vec<String> {
    BRIDGING_AVAILABLE_ASSETS
        .entries()
        .filter_map(|(_token_symbol, chain_map)| {
            chain_map
                .entries()
                .find(|(chain, _)| **chain == chain_id)
                .map(|(_, contract_address)| contract_address.to_string())
        })
        .collect()
}

/// Check is the address is supported bridging asset and return the token symbol and decimals
pub fn find_supported_bridging_asset(chain_id: &str, contract: Address) -> Option<(String, u8)> {
    for (symbol, chain_map) in BRIDGING_AVAILABLE_ASSETS.entries() {
        for (chain, contract_address) in chain_map.entries() {
            if *chain == chain_id && contract == *contract_address {
                return Some((symbol.to_string(), USDC_DECIMALS));
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

/// Check available assets for bridging and return
/// the chain_id, token symbol and contract_address
pub async fn check_bridging_for_erc20_transfer(
    rpc_project_id: String,
    value: U256,
    sender: Address,
    exclude_chain_id: String,
    exclude_contract_address: Address,
) -> Result<Option<BridgingAsset>, RpcError> {
    // Check ERC20 tokens balance for each of supported assets
    let mut contracts_per_chain: HashMap<(String, String), Vec<String>> = HashMap::new();
    for (token_symbol, chain_map) in BRIDGING_AVAILABLE_ASSETS.entries() {
        for (chain_id, contract_address) in chain_map.entries() {
            if *chain_id == exclude_chain_id && *contract_address == exclude_contract_address {
                continue;
            }
            contracts_per_chain
                .entry((token_symbol.to_string(), (*chain_id).to_string()))
                .or_default()
                .push((*contract_address).to_string());
        }
    }
    // Making the check for each chain_id
    for ((token_symbol, chain_id), contracts) in contracts_per_chain {
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
                return Ok(Some(BridgingAsset {
                    chain_id,
                    token_symbol,
                    contract_address,
                    current_balance,
                    // We are supporting only USDC for now which have a fixed decimals
                    decimals: USDC_DECIMALS,
                }));
            }
        }
    }
    Ok(None)
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
) -> Result<(Vec<Erc20AssetChange>, u64), RpcError> {
    // Fill the state overrides for the source address for each of the supported
    // assets on the initial tx chain
    let state_overrides = {
        let mut state_overrides = HashMap::new();
        let assets_contracts =
            get_bridging_assets_contracts_for_chain(&transaction.chain_id.clone());
        let mut account_state = HashMap::new();
        account_state.insert(
            // Since we are using only USDC for the bridging,
            // we can hardcode the storage slot for the contract which is 9
            compute_simulation_storage_slot(transaction.from, 9),
            compute_simulation_balance(99000000000),
        );
        for contract in assets_contracts {
            state_overrides.insert(
                Address::from_str(&contract).unwrap_or_default(),
                account_state.clone(),
            );
        }
        state_overrides
    };

    let simulation_result = &simulation_provider
        .simulate_transaction(
            transaction.chain_id.clone(),
            transaction.from,
            transaction.to,
            transaction.data.clone(),
            state_overrides,
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
