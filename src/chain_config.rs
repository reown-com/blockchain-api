use crate::providers::Priority;
use serde::Serialize;
use std::sync::LazyLock;

// For now, remember to run `cargo run --bin render_chain_config` after updating the config
// TODO in the future, we will pass this via TF variable and generate the chain_config.json file in the CI pipeline
// This however, would increase CD time significantly since ACTIVE_CONFIG is part of this massive crate
// Splitting out only the YAML into separate crate would allow quickly generating the JSON file
pub static ACTIVE_CONFIG: LazyLock<Config> = LazyLock::new(|| Config {
    chains: vec![
        ChainConfig {
            caip2: "eip155:1".to_string(),
            name: "Ethereum".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:10".to_string(),
            name: "Optimism".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:30".to_string(),
            name: "Rootstock".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:31".to_string(),
            name: "Rootstock Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:56".to_string(),
            name: "Binance Smart Chain".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:97".to_string(),
            name: "Binance Smart Chain Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:100".to_string(),
            name: "Gnosis Chain".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:130".to_string(),
            name: "Unichain".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:137".to_string(),
            name: "Polygon".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:146".to_string(),
            name: "Sonic".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:300".to_string(),
            name: "zkSync Era Sepolia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:324".to_string(),
            name: "zkSync Era".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:1101".to_string(),
            name: "Polygon zkEVM".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:1111".to_string(),
            name: "Wemix".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:1112".to_string(),
            name: "Wemix Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:1284".to_string(),
            name: "Moonbeam GLMR".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:1301".to_string(),
            name: "Unichain Sepolia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:1329".to_string(),
            name: "Sei Network".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:2810".to_string(),
            name: "Morph Holesky".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:2818".to_string(),
            name: "Morph Mainnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:5000".to_string(),
            name: "Mantle".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:5003".to_string(),
            name: "Mantle Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:8217".to_string(),
            name: "Kaia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:8453".to_string(),
            name: "Base".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:10143".to_string(),
            name: "Monad Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:17000".to_string(),
            name: "Ethereum Holesky".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:42161".to_string(),
            name: "Arbitrum".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:42220".to_string(),
            name: "Celo".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:43113".to_string(),
            name: "Avalanche Fuji Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:43114".to_string(),
            name: "Avalanche C-Chain".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:57054".to_string(),
            name: "Sonic Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:59144".to_string(),
            name: "Linea".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:80002".to_string(),
            name: "Polygon Amoy".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:80069".to_string(),
            name: "Berachain Bepolia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:80084".to_string(),
            name: "Berachain bArtio".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:80094".to_string(),
            name: "Berachain".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:84532".to_string(),
            name: "Base Sepolia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:421614".to_string(),
            name: "Arbitrum Sepolia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:534352".to_string(),
            name: "Scroll Mainnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:534351".to_string(),
            name: "Scroll Sepolia Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:560048".to_string(),
            name: "Ethereum Hoodi".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:911867".to_string(),
            name: "Odyssey Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:7777777".to_string(),
            name: "Zora".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:11155111".to_string(),
            name: "Ethereum Sepolia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:11155420".to_string(),
            name: "Optimism Sepolia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:999999999".to_string(),
            name: "Zora Sepolia".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:1313161554".to_string(),
            name: "Aurora".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "eip155:1313161555".to_string(),
            name: "Aurora Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "near:mainnet".to_string(),
            name: "Near".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string(),
            name: "Solana".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1".to_string(),
            name: "Solana Devnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "solana:4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z".to_string(),
            name: "Solana Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "bip122:000000000019d6689c085ae165831e93".to_string(),
            name: "Bitcoin".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "bip122:000000000933ea01ad0ee984209779ba".to_string(),
            name: "Bitcoin Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "sui:mainnet".to_string(),
            name: "Sui".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "sui:devnet".to_string(),
            name: "Sui Devnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "sui:testnet".to_string(),
            name: "Sui Testnet".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "stacks:1".to_string(),
            name: "Stacks".to_string(),
            providers: vec![],
        },
        ChainConfig {
            caip2: "stacks:2147483648".to_string(),
            name: "Stacks Testnet".to_string(),
            providers: vec![],
        },
    ],
});

#[derive(Debug, Clone, Serialize)]
pub struct Config {
    pub chains: Vec<ChainConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainConfig {
    pub caip2: String,
    pub name: String,
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderConfig {
    pub url: String,
    #[serde(skip)]
    pub priority: Priority,
}

// TODO
// - env var: RPC_PROXY_RPC_CONFIG_VAR_my_api_key=""
//   - use in-side of `url` via `<my_api_key>`
