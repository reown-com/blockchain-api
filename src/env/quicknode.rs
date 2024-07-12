use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
    tracing::error,
};

#[derive(Debug)]
pub struct QuicknodeConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
    pub chain_subdomains: HashMap<String, String>,
}

impl QuicknodeConfig {
    pub fn new(api_tokens_json: String) -> Self {
        let (supported_chains, chain_subdomains) =
            extract_supported_chains_and_subdomains(api_tokens_json);
        Self {
            supported_chains,
            chain_subdomains,
        }
    }
}

impl ProviderConfig for QuicknodeConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
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
        (
            "eip155:324",
            ("snowy-chaotic-hill.zksync-mainnet", Priority::High),
        ),
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
