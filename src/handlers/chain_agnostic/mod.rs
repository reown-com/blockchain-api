use phf::phf_map;

pub mod check;

/// Available assets for Bridging
pub static BRIDGING_AVAILABLE_ASSETS: phf::Map<&'static str, phf::Map<&'static str, &'static str>> = phf_map! {
  "USDC" => phf_map! {
      // Base
      "eip155:56" => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
      // Optimism
      "eip155:10" => "0x0b2c639c533813f4aa9d7837caf62653d097ff85",
      // Arbitrum
      "eip155:42161" => "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
  },
};
