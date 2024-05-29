use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
    tracing::error,
};

#[derive(Debug)]
pub struct GetBlockConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl GetBlockConfig {
    pub fn new(access_tokens_json: String) -> Self {
        Self {
            supported_chains: extract_supported_chains(access_tokens_json),
        }
    }
}

impl ProviderConfig for GetBlockConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::GetBlock
    }
}

fn extract_supported_chains(access_tokens_json: String) -> HashMap<String, (String, Weight)> {
    let access_tokens: HashMap<String, String> = match serde_json::from_str(&access_tokens_json) {
        Ok(tokens) => tokens,
        Err(_) => {
            error!(
                "Failed to parse JSON with API access tokens for GetBlock provider. Using empty \
                 tokens."
            );
            return HashMap::new();
        }
    };

    // Keep in-sync with SUPPORTED_CHAINS.md
    let supported_chain_ids = HashMap::from([
        ("eip155:1", Priority::Low),
        ("eip155:56", Priority::Low),
        ("eip155:137", Priority::Normal),
        ("eip155:324", Priority::Normal),
        ("eip155:17000", Priority::Normal),
        ("eip155:11155111", Priority::Normal),
        ("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", Priority::Normal),
    ]);

    let access_tokens_with_weights: HashMap<String, (String, Weight)> = supported_chain_ids
        .iter()
        .filter_map(|(&key, &weight)| {
            if let Some(token) = access_tokens.get(key) {
                match Weight::new(weight) {
                    Ok(weight) => Some((key.to_string(), (token.to_string(), weight))),
                    Err(_) => {
                        error!(
                            "Failed to create Weight for key {} in GetBlock provider",
                            key
                        );
                        None
                    }
                }
            } else {
                error!(
                    "GetBlock provider API access token for {} is not present, skipping it",
                    key
                );
                None
            }
        })
        .collect();

    access_tokens_with_weights
}
