use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
    tracing::error,
};

#[derive(Debug)]
pub struct QuicknodeConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
    pub supported_ws_chains: HashMap<String, (String, Weight)>,
    pub chain_subdomains: HashMap<String, String>,
}

impl QuicknodeConfig {
    pub fn new(api_tokens_json: String) -> Self {
        let (supported_chains, chain_subdomains) =
            extract_supported_chains_and_subdomains(api_tokens_json.clone());
        let supported_ws_chains = extract_ws_supported_chains_and_subdomains(api_tokens_json);
        Self {
            supported_chains,
            supported_ws_chains,
            chain_subdomains,
        }
    }
}

impl ProviderConfig for QuicknodeConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_ws_chains
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Quicknode
    }
}

fn extract_supported_chains_and_subdomains(
    access_tokens_json: String,
) -> (HashMap<String, (String, Weight)>, HashMap<String, String>) {
    let access_tokens: HashMap<String, String> = match serde_json::from_str(&access_tokens_json) {
        Ok(tokens) => tokens,
        Err(_) => {
            error!(
                "Failed to parse JSON with API access tokens for QuickNode provider. Using empty \
                 tokens."
            );
            return (HashMap::new(), HashMap::new());
        }
    };

    // Keep in-sync with SUPPORTED_CHAINS.md
    // Supported chains list format: chain ID, subdomain, priority
    let supported_chain_ids = HashMap::from([
        ("eip155:1", ("ancient-snowy-snow", Priority::Minimal)),
        (
            "eip155:324",
            ("snowy-chaotic-hill.zksync-mainnet", Priority::Low),
        ),
        (
            "eip155:1101",
            ("clean-few-meme.zkevm-mainnet", Priority::Low),
        ),
        (
            "eip155:42161",
            ("divine-special-snowflake.arbitrum-mainnet", Priority::Low),
        ),
        (
            "eip155:80084",
            ("frequent-capable-putty.bera-bartio", Priority::Low),
        ),
        (
            "eip155:10",
            ("convincing-dawn-smoke.optimism", Priority::Low),
        ),
        (
            "eip155:8453",
            ("indulgent-methodical-emerald.base-mainnet", Priority::Low),
        ),
        ("eip155:56", ("muddy-compatible-general.bsc", Priority::Low)),
        (
            "eip155:8217",
            ("sleek-solitary-telescope.kaia-mainnet", Priority::Low),
        ),
        (
            "eip155:421614",
            ("thrumming-quaint-flower.arbitrum-sepolia", Priority::Low),
        ),
        (
            "eip155:80094",
            ("convincing-prettiest-wind.bera-mainnet", Priority::Low),
        ),
        (
            "eip155:11155111",
            ("crimson-spring-wind.ethereum-sepolia", Priority::Low),
        ),
        (
            "eip155:137",
            ("chaotic-wandering-fire.matic", Priority::Low),
        ),
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
            ("indulgent-thrumming-bush.solana-mainnet", Priority::Minimal),
        ),
        (
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
            ("wild-palpable-rain.solana-devnet", Priority::Normal),
        ),
        (
            "solana:4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z",
            ("winter-flashy-glade.solana-testnet", Priority::Normal),
        ),
        (
            "bip122:000000000019d6689c085ae165831e93",
            ("warmhearted-multi-mound.btc", Priority::Normal),
        ),
        (
            "bip122:000000000933ea01ad0ee984209779ba",
            ("newest-lively-research.btc-testnet", Priority::Normal),
        ),
    ]);

    let access_tokens_with_weights: HashMap<String, (String, Weight)> = supported_chain_ids
        .iter()
        .filter_map(|(&key, (_, weight))| {
            if let Some(token) = access_tokens.get(key) {
                match Weight::new(*weight) {
                    Ok(weight) => Some((key.to_string(), (token.to_string(), weight))),
                    Err(_) => {
                        error!(
                            "Failed to create Weight for key {} in QuickNode provider",
                            key
                        );
                        None
                    }
                }
            } else {
                error!(
                    "QuickNode provider API access token for {} is not present, skipping it",
                    key
                );
                None
            }
        })
        .collect();
    let chain_ids_subdomains: HashMap<String, String> = supported_chain_ids
        .iter()
        .map(|(&key, (subdomain, _))| (key.to_string(), subdomain.to_string()))
        .collect();

    (access_tokens_with_weights, chain_ids_subdomains)
}

fn extract_ws_supported_chains_and_subdomains(
    access_tokens_json: String,
) -> HashMap<String, (String, Weight)> {
    let access_tokens: HashMap<String, String> = match serde_json::from_str(&access_tokens_json) {
        Ok(tokens) => tokens,
        Err(_) => {
            error!(
                "Failed to parse JSON with API ws access tokens for QuickNode provider. Using empty \
                 tokens."
            );
            return HashMap::new();
        }
    };

    // Keep in-sync with SUPPORTED_CHAINS.md
    // Supported chains list format: chain ID, subdomain, priority
    let supported_chain_ids = HashMap::from([
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
            ("indulgent-thrumming-bush.solana-mainnet", Priority::Normal),
        ),
        (
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
            ("wild-palpable-rain.solana-devnet", Priority::Normal),
        ),
        (
            "solana:4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z",
            ("winter-flashy-glade.solana-testnet", Priority::Normal),
        ),
    ]);

    let access_tokens_with_weights: HashMap<String, (String, Weight)> = supported_chain_ids
        .iter()
        .filter_map(|(&key, (_, weight))| {
            if let Some(token) = access_tokens.get(key) {
                match Weight::new(*weight) {
                    Ok(weight) => Some((key.to_string(), (token.to_string(), weight))),
                    Err(_) => {
                        error!(
                            "Failed to create Weight for key {} in QuickNode provider",
                            key
                        );
                        None
                    }
                }
            } else {
                error!(
                    "QuickNode provider API ws access token for {} is not present, skipping it",
                    key
                );
                None
            }
        })
        .collect();

    access_tokens_with_weights
}
