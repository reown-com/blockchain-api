use {
    alloy::primitives::{address, Address},
    phf::phf_map,
};

pub struct AssetMetadata {
    pub decimals: u8,
}

/// Asset simulation parameters to override the asset's balance state
pub struct SimulationParams {
    /// Asset contract balance storage slot number per chain
    pub balance_storage_slots: &'static phf::Map<&'static str, u64>,
    /// Balance override for the asset
    pub balance: u128,
}

pub struct AssetEntry {
    pub metadata: AssetMetadata,
    pub simulation: SimulationParams,
    /// Asset contracts per CAIP-2 chain ID
    pub contracts: &'static phf::Map<&'static str, Address>,
}

static USDC_CONTRACTS: phf::Map<&'static str, Address> = phf_map! {
    // Optimism
    "eip155:10" => address!("0b2c639c533813f4aa9d7837caf62653d097ff85"),
    // Base
    "eip155:8453" => address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
    // Arbitrum
    "eip155:42161" => address!("af88d065e77c8cC2239327C5EDb3A432268e5831"),
};

static USDT_CONTRACTS: phf::Map<&'static str, Address> = phf_map! {
    // Optimism
    "eip155:10" => address!("94b008aA00579c1307B0EF2c499aD98a8ce58e58"),
    // Arbitrum
    "eip155:42161" => address!("Fd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9"),
};

static USDS_CONTRACTS: phf::Map<&'static str, Address> = phf_map! {
    // Optimism
    "eip155:10" => address!("DA10009cBd5D07dd0CeCc66161FC93D7c9000da1"),
    // Arbitrum
    "eip155:42161" => address!("DA10009cBd5D07dd0CeCc66161FC93D7c9000da1"),
};

pub static BRIDGING_ASSETS: phf::Map<&'static str, AssetEntry> = phf_map! {
    "USDC" => AssetEntry {
        metadata: AssetMetadata {
            decimals: 6,
        },
        simulation: SimulationParams {
            // Must be in sync with the `USDC_CONTRACTS` from above
            balance_storage_slots: &phf_map! {
                "eip155:10" => 9u64,
                "eip155:8453" => 9u64,
                "eip155:42161" => 9u64,
            },
            balance: 99000000000,
        },
        contracts: &USDC_CONTRACTS,
    },
    "USDT" => AssetEntry {
        metadata: AssetMetadata {
            decimals: 6,
        },
        simulation: SimulationParams {
            // Must be in sync with the `USDT_CONTRACTS` from above
            balance_storage_slots: &phf_map! {
                "eip155:10" => 0u64,
                "eip155:42161" => 51u64,
            },
            balance: 99000000000,
        },
        contracts: &USDT_CONTRACTS,
    },
    "USDS" => AssetEntry {
        metadata: AssetMetadata {
            decimals: 18,
        },
        simulation: SimulationParams {
            // Must be in sync with the `USDS_CONTRACTS` from above
            balance_storage_slots: &phf_map! {
                "eip155:10" => 2u64,
                "eip155:42161" => 2u64,
            },
            balance: 99000000000000000000000,
        },
        contracts: &USDS_CONTRACTS,
    },
};
