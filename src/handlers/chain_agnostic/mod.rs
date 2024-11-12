use {
    crate::{error::RpcError, handlers::MessageSource, utils::crypto::get_erc20_contract_balance},
    alloy::primitives::{Address, U256},
    ethers::types::H160 as EthersH160,
    phf::phf_map,
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, str::FromStr},
};

pub mod route;
pub mod status;

/// How much to multiply the amount by when bridging to cover bridging differences
pub const BRIDGING_AMOUNT_MULTIPLIER: i8 = 5; // 5%

/// Available assets for Bridging
pub static BRIDGING_AVAILABLE_ASSETS: phf::Map<&'static str, phf::Map<&'static str, &'static str>> = phf_map! {
  "USDC" => phf_map! {
      // Optimism
      "eip155:10" => "0x0b2c639c533813f4aa9d7837caf62653d097ff85",
      // Base
      "eip155:8453" => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
      // Arbitrum
      "eip155:42161" => "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
  },
};

/// The status polling interval in ms for the client
pub const STATUS_POLLING_INTERVAL: usize = 3000; // 3 seconds

/// Serialized bridging request item schema to store it in the IRN database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBridgingItem {
    created_at: usize,
    chain_id: String,
    wallet: Address,
    contract: Address,
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

/// Check is the address is supported bridging asset
pub fn is_supported_bridging_asset(chain_id: String, contract: Address) -> bool {
    BRIDGING_AVAILABLE_ASSETS.entries().any(|(_, chain_map)| {
        chain_map.entries().any(|(chain, contract_address)| {
            *chain == chain_id
                && contract == Address::from_str(contract_address).unwrap_or_default()
        })
    })
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

/// Check available assets for bridging and return
/// the chain_id, token symbol and contract_address
pub async fn check_bridging_for_erc20_transfer(
    rpc_project_id: String,
    value: U256,
    sender: Address,
) -> Result<Option<(String, String, Address)>, RpcError> {
    // Check ERC20 tokens balance for each of supported assets
    let mut contracts_per_chain: HashMap<(String, String), Vec<String>> = HashMap::new();
    for (token_symbol, chain_map) in BRIDGING_AVAILABLE_ASSETS.entries() {
        for (chain_id, contract_address) in chain_map.entries() {
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
        for (contract, balance) in erc20_balances {
            if balance >= value {
                return Ok(Some((chain_id, token_symbol, contract)));
            }
        }
    }
    Ok(None)
}
