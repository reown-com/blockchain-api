mod infura;
mod pokt;

use crate::handlers::RPCQueryParams;
use async_trait::async_trait;
use hyper::Body;
use hyper::Error;
use hyper::Response;
pub use infura::InfuraProvider;
pub use pokt::PoktProvider;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct ProviderRepository {
    map: HashMap<String, Arc<dyn RPCProvider>>,
}

impl ProviderRepository {
    pub fn get_provider_for_chain_id(&self, chain: &str) -> Option<&Arc<dyn RPCProvider>> {
        self.map.values().into_iter()
            .find(|provider| provider.supports_caip_chainid(chain))
    }
    pub fn add_provider(&mut self, chain: String, provider: Arc<dyn RPCProvider>) {
        self.map.insert(chain, provider);
    }
}

#[async_trait]
pub trait RPCProvider: Send + Sync {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        xpath: warp::path::FullPath,
        query_params: RPCQueryParams,
        headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> Result<Response<Body>, Error>;

    fn supports_caip_chainid(&self, chain_id: &str) -> bool;
}
