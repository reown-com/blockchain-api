use {
    super::ProviderConfig,
    crate::providers::{Priority, Weight},
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct TrongridConfig {
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Default for TrongridConfig {
    fn default() -> Self {
        Self {
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for TrongridConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        self.supported_chains
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Trongrid
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    HashMap::from([(
        "tron:0xcd8690dc".into(),
        (
            "https://nile.trongrid.io/jsonrpc".into(),
            Weight::new(Priority::Normal).unwrap(),
        ),
    )])
}

