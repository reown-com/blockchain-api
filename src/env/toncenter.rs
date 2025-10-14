use {
    super::BalanceProviderConfig,
    crate::{
        providers::{Priority, Weight},
        utils::crypto::CaipNamespaces,
    },
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct ToncenterConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub supported_namespaces: HashMap<CaipNamespaces, Weight>,
}

impl ToncenterConfig {
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        Self {
            api_url,
            api_key,
            supported_namespaces: default_supported_namespaces(),
        }
    }
}

impl BalanceProviderConfig for ToncenterConfig {
    fn supported_namespaces(self) -> HashMap<CaipNamespaces, Weight> {
        self.supported_namespaces
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Toncenter
    }
}

fn default_supported_namespaces() -> HashMap<CaipNamespaces, Weight> {
    HashMap::from([(CaipNamespaces::Ton, Weight::new(Priority::Normal).unwrap())])
}
