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
        Self {
            supported_chains: extract_supported_chains(api_tokens_json),
            chain_subdomains: default_chain_subdomains(),
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

fn extract_supported_chains(access_tokens_json: String) -> HashMap<String, (String, Weight)> {
    let access_tokens: HashMap<String, String> = match serde_json::from_str(&access_tokens_json) {
        Ok(tokens) => tokens,
        Err(_) => {
            error!(
                "Failed to parse JSON with API access tokens for QuickNode provider. Using empty \
                 tokens."
            );
            return HashMap::new();
        }
    };

    // Keep in-sync with SUPPORTED_CHAINS.md
    let supported_chain_ids = HashMap::from([
        ("eip155:324", Priority::High),
        ("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", Priority::Normal),
        ("solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1", Priority::Normal),
        ("solana:4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z", Priority::Normal),
    ]);

    let access_tokens_with_weights: HashMap<String, (String, Weight)> = supported_chain_ids
        .iter()
        .filter_map(|(&key, weight)| {
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

    access_tokens_with_weights
}

fn default_chain_subdomains() -> HashMap<String, String> {
    HashMap::from([
        // zkSync
        (
            "eip155:324".into(),
            "snowy-chaotic-hill.zksync-mainnet".into(),
        ),
        // Solana Mainnet
        (
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".into(),
            "indulgent-thrumming-bush.solana-mainnet".into(),
        ),
        // Solana Devnet
        (
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1".into(),
            "wild-palpable-rain.solana-devnet".into(),
        ),
        // Solana Testnet
        (
            "solana:4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z".into(),
            "winter-flashy-glade.solana-testnet".into(),
        ),
    ])
}
