use {
    super::{BalanceProviderConfig, ProviderConfig},
    crate::{
        providers::{Priority, Weight},
        utils::crypto::CaipNamespaces,
    },
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct ToncenterV3Config {
    pub api_url: String,
    pub api_key: Option<String>,
    pub supported_namespaces: HashMap<CaipNamespaces, Weight>,
}

impl ToncenterV3Config {
    pub fn new(api_url: String, api_key: Option<String>) -> Self {
        Self {
            api_url,
            api_key,
            supported_namespaces: default_supported_namespaces(),
        }
    }
}

impl BalanceProviderConfig for ToncenterV3Config {
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

#[derive(Debug)]
pub struct ToncenterV2Config {
    pub api_key: Option<String>,
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl ToncenterV2Config {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key,
            supported_chains: default_supported_chains(),
        }
    }
}

impl ProviderConfig for ToncenterV2Config {
    fn supported_chains(self) -> HashMap<String, (String, Weight)> {
        default_supported_chains()
    }

    fn supported_ws_chains(self) -> HashMap<String, (String, Weight)> {
        HashMap::new()
    }

    fn provider_kind(&self) -> crate::providers::ProviderKind {
        crate::providers::ProviderKind::Toncenter
    }
}

fn default_supported_chains() -> HashMap<String, (String, Weight)> {
    // Keep in-sync with SUPPORTED_CHAINS.md
    HashMap::from([
        // TON Mainnet
        (
            "ton:-239".into(),
            // Don't use a subdomain for the mainnet
            (
                "toncenter.com".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
        // TON Testnet
        (
            "ton:-3".into(),
            (
                "testnet.toncenter.com".into(),
                Weight::new(Priority::Normal).unwrap(),
            ),
        ),
    ])
}
