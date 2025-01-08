use {
    super::BalanceProviderConfig,
    crate::{
        providers::{Priority, Weight},
        utils::crypto::CaipNamespaces,
    },
    std::collections::HashMap,
};

pub struct SolScanConfig {
    pub api_key: String,
    pub supported_namespaces: HashMap<CaipNamespaces, Weight>,
}

impl SolScanConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            supported_namespaces: default_supported_namespaces(),
        }
    }
}

impl BalanceProviderConfig for SolScanConfig {
    fn supported_namespaces(self) -> HashMap<CaipNamespaces, Weight> {
        self.supported_namespaces
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::SolScan
    }
}

fn default_supported_namespaces() -> HashMap<CaipNamespaces, Weight> {
    HashMap::from([(
        CaipNamespaces::Solana,
        Weight::new(Priority::Normal).unwrap(),
    )])
}
