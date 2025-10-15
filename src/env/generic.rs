use {
    super::ProviderConfig,
    crate::{chain_config, providers::Weight},
    base64::Engine,
    std::collections::HashMap,
};

#[derive(Debug, Clone)]
pub struct GenericConfig {
    pub caip2: String,
    pub name: String,
    pub provider: chain_config::ProviderConfig,
}

impl ProviderConfig for GenericConfig {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::from([(
            self.caip2,
            (
                self.provider.url,
                Weight::new(self.provider.priority).unwrap(),
            ),
        )])
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        let deterministic_id = base64::engine::general_purpose::STANDARD
            .encode(format!("{}-{}", self.caip2, self.provider.url));
        crate::providers::ProviderKind::Generic(deterministic_id)
    }
}
