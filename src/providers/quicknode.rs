use {
    super::{
        Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory, RpcQueryParams,
        RpcWsProvider, WS_PROXY_TASK_METRICS,
    },
    crate::{
        env::QuicknodeConfig,
        error::{RpcError, RpcResult},
        ws,
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    axum::extract::ws::WebSocketUpgrade,
    
    hyper::http,
    std::collections::HashMap,
    wc::future::FutureExt,
};

#[derive(Debug)]
pub struct QuicknodeProvider {
    pub client: reqwest::Client,
    pub supported_chains: HashMap<String, String>,
    pub chain_subdomains: HashMap<String, String>,
}

impl Provider for QuicknodeProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Quicknode
    }
}

#[async_trait]
impl RateLimited for QuicknodeProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for QuicknodeProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let token = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        // Get the chain subdomain
        let chain_subdomain =
            self.chain_subdomains
                .get(chain_id)
                .ok_or(RpcError::InvalidConfiguration(format!(
                    "Quicknode subdomain not found for chainId: {chain_id}"
                )))?;
        let uri = format!("https://{chain_subdomain}.quiknode.pro/{token}");

        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;
        let mut response = (status, body).into_response();
        response
            .headers_mut()
            .insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok(response)
    }
}

impl RpcProviderFactory<QuicknodeConfig> for QuicknodeProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &QuicknodeConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        QuicknodeProvider {
            client: forward_proxy_client,
            supported_chains,
            chain_subdomains: provider_config.chain_subdomains.clone(),
        }
    }
}

#[derive(Debug)]
pub struct QuicknodeWsProvider {
    pub supported_chains: HashMap<String, String>,
    pub chain_subdomains: HashMap<String, String>,
}

impl Provider for QuicknodeWsProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Quicknode
    }
}

#[async_trait]
impl RpcWsProvider for QuicknodeWsProvider {
    #[tracing::instrument(skip_all, fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response> {
        let chain_id = &query_params.chain_id;
        let project_id = query_params.project_id;
        let token = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let chain_subdomain =
            self.chain_subdomains
                .get(chain_id)
                .ok_or(RpcError::InvalidConfiguration(format!(
                    "Quicknode wss subdomain not found for chainId: {chain_id}"
                )))?;
        let uri = format!("wss://{chain_subdomain}.quiknode.pro/{token}");
        let (websocket_provider, _) = async_tungstenite::tokio::connect_async(uri)
            .await
            .map_err(|e| RpcError::AxumTungstenite(Box::new(e)))?;

        Ok(ws.on_upgrade(move |socket| {
            ws::proxy(project_id, socket, websocket_provider)
                .with_metrics(WS_PROXY_TASK_METRICS.with_name("quicknode"))
        }))
    }
}

#[async_trait]
impl RateLimited for QuicknodeWsProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

impl RpcProviderFactory<QuicknodeConfig> for QuicknodeWsProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &QuicknodeConfig) -> Self {
        let supported_chains: HashMap<String, String> = provider_config
            .supported_ws_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();
        let chain_subdomains = provider_config.chain_subdomains.clone();

        QuicknodeWsProvider {
            supported_chains,
            chain_subdomains,
        }
    }
}
