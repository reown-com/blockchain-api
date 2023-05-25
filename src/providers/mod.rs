use {
    crate::env::ProviderConfig,
    axum::response::Response,
    axum_tungstenite::WebSocketUpgrade,
    rand::{distributions::WeightedIndex, prelude::Distribution, rngs::OsRng},
    std::{fmt::Debug, hash::Hash, sync::Arc},
    tracing::info,
};

mod binance;
mod infura;
mod omnia;
mod pokt;
mod publicnode;
mod zksync;

use {
    crate::{error::RpcResult, handlers::RpcQueryParams},
    async_trait::async_trait,
    std::{collections::HashMap, fmt::Display},
};
pub use {
    binance::BinanceProvider,
    infura::{InfuraProvider, InfuraWsProvider},
    omnia::OmniatechProvider,
    pokt::PoktProvider,
    publicnode::PublicnodeProvider,
    zksync::ZKSyncProvider,
};

#[derive(Default, Debug)]
pub struct ProviderRepository {
    providers: HashMap<ProviderKind, Arc<dyn RpcProvider>>,
    ws_providers: HashMap<ProviderKind, Arc<dyn RpcWsProvider>>,
    // TODO: create newtype for ChainId
    weight_resolver: HashMap<String, Vec<(ProviderKind, Weight)>>,
    ws_weight_resolver: HashMap<String, Vec<(ProviderKind, Weight)>>,
}

impl ProviderRepository {
    pub fn get_provider_for_chain_id(&self, chain_id: &str) -> Option<Arc<dyn RpcProvider>> {
        let Some(providers) = self.weight_resolver.get(chain_id) else {return None};

        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| weight.value()).collect();
        let dist = WeightedIndex::new(weights).unwrap();
        let provider = &providers[dist.sample(&mut OsRng)].0;

        self.providers.get(provider).cloned()
    }

    pub fn get_ws_provider_for_chain_id(&self, chain_id: &str) -> Option<Arc<dyn RpcWsProvider>> {
        let Some(providers) = self.ws_weight_resolver.get(chain_id) else {return None};

        if providers.is_empty() {
            return None;
        }

        let weights: Vec<_> = providers.iter().map(|(_, weight)| weight.value()).collect();
        let dist = WeightedIndex::new(weights).unwrap();
        let provider = &providers[dist.sample(&mut OsRng)].0;

        self.ws_providers.get(provider).cloned()
    }

    pub fn add_ws_provider(&mut self, provider: impl ProviderConfig) {
        // provider
        //     .supported_caip_chains()
        //     .into_iter()
        //     .for_each(|chain| {
        //         self.ws_map
        //             .entry(chain.chain_id)
        //             .or_insert_with(Vec::new)
        //             .push((provider.clone(), chain.weight));
        //     });
    }

    pub fn add_provider(&mut self, provider_config: impl ProviderConfig) {
        // Create new provider, take config as argument
        // Store the provider under ProviderKind => Provider (enum => struct)
        // Strip weights from the provider, only keep mapping
        // Build weighted map chainId => ProviderKind
        // This way we don't need cloning.
        // We consume the config, so we can take the weights and put them in the
        // map As we never clone the Weights, we can just update them in
        // the map provider
        //     .supported_caip_chains()
        //     .into_iter()
        //     .for_each(|chain| {
        //         self.map
        //             .entry(chain.chain_id)
        //             .or_insert_with(Vec::new)
        //             .push((provider.clone(), chain.weight));
        //     });
    }

    pub fn update_weights(&self) {
        info!("Updating weights");
        // self.map.iter().for_each(|(_, providers)| {
        //     providers.iter().for_each(|(_, weight)| {
        //         weight.0.store(3, std::sync::atomic::Ordering::SeqCst);
        //     });
        // });
        // self.weight_resolver.
    }
}

// TODO: Find better name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Infura,
    Pokt,
    Binance,
    ZKSync,
    Publicnode,
    Omniatech,
}

impl Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ProviderKind::Infura => "Infura",
            ProviderKind::Pokt => "Pokt",
            ProviderKind::Binance => "Binance",
            ProviderKind::ZKSync => "zkSync",
            ProviderKind::Publicnode => "Publicnode",
            ProviderKind::Omniatech => "Omniatech",
        })
    }
}

#[async_trait]
pub trait RpcProvider: Send + Sync + Provider + Debug {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        xpath: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response>;
}

#[async_trait]
pub trait RpcWsProvider: Send + Sync + Provider + Debug {
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response>;
}

#[derive(Debug)]
pub struct Weight(pub std::sync::atomic::AtomicU32);

impl Weight {
    pub fn value(&self) -> u32 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

// TODO: This is should not be Clone ever.
// Cloning it makes it possible that updates to the weight are not reflected in
// the map
impl Clone for Weight {
    fn clone(&self) -> Self {
        let atomic =
            std::sync::atomic::AtomicU32::new(self.0.load(std::sync::atomic::Ordering::SeqCst));
        Self(atomic)
    }
}

#[derive(Debug)]
pub struct SupportedChain {
    pub chain_id: String,
    pub weight: Weight,
}

pub trait Provider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool;

    fn supported_caip_chains(&self) -> Vec<String>;

    fn provider_kind(&self) -> ProviderKind;
}
