use {
    super::{
        Provider,
        ProviderKind,
        RateLimited,
        RateLimitedData,
        RpcProvider,
        RpcProviderFactory,
        RpcQueryParams,
        RpcWsProvider,
    },
    crate::{
        env::InfuraConfig,
        error::{RpcError, RpcResult},
        ws,
    },
    async_trait::async_trait,
    axum::response::{IntoResponse, Response},
    axum_tungstenite::WebSocketUpgrade,
    hyper::{client::HttpConnector, http, Client},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct InfuraProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub project_id: String,
    pub supported_chains: HashMap<String, String>,
}

#[derive(Debug)]
pub struct InfuraWsProvider {
    pub project_id: String,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for InfuraWsProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Infura
    }
}

impl RateLimited for InfuraWsProvider {
    fn is_rate_limited(response: RateLimitedData) -> bool
    where
        Self: Sized,
    {
        let RateLimitedData::Response(response) = response else {return false};
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcWsProvider for InfuraWsProvider {
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(&query_params.chain_id.to_lowercase())
            .ok_or(RpcError::ChainNotFound)?;

        let project_id = query_params.project_id;

        let uri = format!("wss://{}.infura.io/ws/v3/{}", chain, self.project_id);

        let (websocket_provider, _) = async_tungstenite::tokio::connect_async(uri).await?;

        Ok(ws.on_upgrade(move |socket| ws::proxy(project_id, socket, websocket_provider)))
    }
}

impl Provider for InfuraProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Infura
    }
}

impl RateLimited for InfuraProvider {
    fn is_rate_limited(response: RateLimitedData) -> bool
    where
        Self: Sized,
    {
        let RateLimitedData::Response(response) = response else {return false};
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for InfuraProvider {
    async fn proxy(
        &self,
        method: hyper::http::Method,
        _path: axum::extract::MatchedPath,
        query_params: RpcQueryParams,
        _headers: hyper::http::HeaderMap,
        body: hyper::body::Bytes,
    ) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(&query_params.chain_id.to_lowercase())
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://{}.infura.io/v3/{}", chain, self.project_id);

        let hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?.into_response();

        if Self::is_rate_limited(RateLimitedData::Response(&response)) {
            return Err(RpcError::Throttled);
        }

        Ok(response)
    }
}

impl RpcProviderFactory<InfuraConfig> for InfuraProvider {
    fn new(provider_config: &InfuraConfig) -> Self {
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        InfuraProvider {
            client: forward_proxy_client,
            supported_chains,
            project_id: provider_config.project_id.clone(),
        }
    }
}

impl RpcProviderFactory<InfuraConfig> for InfuraWsProvider {
    fn new(provider_config: &InfuraConfig) -> Self {
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        InfuraWsProvider {
            supported_chains,
            project_id: provider_config.project_id.clone(),
        }
    }
}
