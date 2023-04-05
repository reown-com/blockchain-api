use {axum::response::Response, axum_tungstenite::WebSocketUpgrade};

mod binance;
mod infura;
mod pokt;
mod zksync;

use {
    crate::{error::RpcResult, handlers::RpcQueryParams},
    async_trait::async_trait,
    std::{collections::HashMap, fmt::Display, sync::Arc},
};
pub use {
    binance::BinanceProvider,
    infura::{InfuraProvider, InfuraWsProvider},
    pokt::PoktProvider,
    zksync::ZKSyncProvider,
};

#[derive(Default, Clone)]
pub struct ProviderRepository {
    map: HashMap<String, Arc<dyn RpcProvider>>,
    ws_map: HashMap<String, Arc<dyn RpcWsProvider>>,
}

impl ProviderRepository {
    pub fn get_provider_for_chain_id(&self, chain_id: &str) -> Option<&Arc<dyn RpcProvider>> {
        self.map.get(chain_id)
    }

    pub fn get_ws_provider_for_chain_id(&self, chain_id: &str) -> Option<&Arc<dyn RpcWsProvider>> {
        self.ws_map.get(chain_id)
    }

    pub fn add_ws_provider(&mut self, _provider_name: String, provider: Arc<dyn RpcWsProvider>) {
        provider
            .supported_caip_chainids()
            .into_iter()
            .for_each(|chain| {
                self.ws_map.insert(chain, provider.clone());
            });
    }

    pub fn add_provider(&mut self, _provider_name: String, provider: Arc<dyn RpcProvider>) {
        provider
            .supported_caip_chainids()
            .into_iter()
            .for_each(|chain| {
                self.map.insert(chain, provider.clone());
            });
    }
}

pub enum ProviderKind {
    Infura,
    Pokt,
    Binance,
    ZKSync,
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ProviderKind::Infura => "Infura",
            ProviderKind::Pokt => "Pokt",
            ProviderKind::Binance => "Binance",
            ProviderKind::ZKSync => "zkSync",
        })
    }
}

#[async_trait]
pub trait RpcProvider: Send + Sync {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        xpath: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response>;

    fn supports_caip_chainid(&self, chain_id: &str) -> bool;

    fn supported_caip_chainids(&self) -> Vec<String>;

    fn provider_kind(&self) -> ProviderKind;

    fn project_id(&self) -> &str;
}

#[async_trait]
pub trait RpcWsProvider: Send + Sync {
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response>;

    fn supports_caip_chainid(&self, chain_id: &str) -> bool;

    fn supported_caip_chainids(&self) -> Vec<String>;

    fn provider_kind(&self) -> ProviderKind;

    fn project_id(&self) -> &str;
}
