use {
    super::{
        Provider,
        ProviderKind,
        RateLimited,
        RpcProvider,
        RpcProviderFactory,
        RpcQueryParams,
        RpcWsProvider,
        WS_PROXY_TASK_METRICS,
    },
    crate::{
        env::ZoraConfig,
        error::{RpcError, RpcResult},
        ws,
    },
    async_trait::async_trait,
    axum::response::{IntoResponse, Response},
    axum_tungstenite::WebSocketUpgrade,
    hyper::{client::HttpConnector, http, Client, Method},
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
    wc::future::FutureExt,
};

#[derive(Debug)]
pub struct ZoraProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub supported_chains: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ZoraWsProvider {
    pub supported_chains: HashMap<String, String>,
}

impl Provider for ZoraWsProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Zora
    }
}

#[async_trait]
impl RateLimited for ZoraWsProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool
    where
        Self: Sized,
    {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcWsProvider for ZoraWsProvider {
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&query_params.chain_id.to_lowercase())
            .ok_or(RpcError::ChainNotFound)?;

        let project_id = query_params.project_id;

        let (websocket_provider, _) = async_tungstenite::tokio::connect_async(uri).await?;

        Ok(ws.on_upgrade(move |socket| {
            ws::proxy(project_id, socket, websocket_provider)
                .with_metrics(WS_PROXY_TASK_METRICS.with_name("Zora"))
        }))
    }
}

impl Provider for ZoraProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Zora
    }
}

#[async_trait]
impl RateLimited for ZoraProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool
    where
        Self: Sized,
    {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for ZoraProvider {
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?.into_response();

        Ok(response)
    }
}

impl RpcProviderFactory<ZoraConfig> for ZoraProvider {
    fn new(provider_config: &ZoraConfig) -> Self {
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        ZoraProvider {
            client: forward_proxy_client,
            supported_chains,
        }
    }
}

impl RpcProviderFactory<ZoraConfig> for ZoraWsProvider {
    fn new(provider_config: &ZoraConfig) -> Self {
        let supported_chains: HashMap<String, String> = provider_config
            .supported_ws_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        ZoraWsProvider { supported_chains }
    }
}
