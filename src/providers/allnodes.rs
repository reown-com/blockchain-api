use {
    super::{
        Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory, RpcQueryParams,
        RpcWsProvider,
    },
    crate::{
        env::AllnodesConfig,
        error::{RpcError, RpcResult},
        ws,
    },
    async_trait::async_trait,
    axum::{
        extract::ws::WebSocketUpgrade,
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::http,
    std::collections::HashMap,
    wc::metrics::{future_metrics, FutureExt},
};

#[derive(Debug)]
pub struct AllnodesProvider {
    pub client: reqwest::Client,
    pub supported_chains: HashMap<String, String>,
    pub api_key: String,
}

#[derive(Debug)]
pub struct AllnodesWsProvider {
    pub supported_chains: HashMap<String, String>,
    pub api_key: String,
}

impl Provider for AllnodesWsProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Allnodes
    }
}

#[async_trait]
impl RpcWsProvider for AllnodesWsProvider {
    #[tracing::instrument(skip_all, fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(&query_params.chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let project_id = query_params.project_id;
        let uri = format!("wss://{}.allnodes.me:8546/{}", chain, &self.api_key);
        let (websocket_provider, _) = async_tungstenite::tokio::connect_async(uri)
            .await
            .map_err(|e| RpcError::WebSocketError(e.to_string()))?;

        Ok(ws.on_upgrade(move |socket| {
            ws::proxy(project_id, socket, websocket_provider)
                .with_metrics(future_metrics!("ws_proxy_task", "name" => "allnodes"))
        }))
    }
}

#[async_trait]
impl RateLimited for AllnodesWsProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool
    where
        Self: Sized,
    {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

impl Provider for AllnodesProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Allnodes
    }
}

#[async_trait]
impl RateLimited for AllnodesProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for AllnodesProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://{}.allnodes.me:8545/{}", chain, &self.api_key);

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

impl RpcProviderFactory<AllnodesConfig> for AllnodesProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &AllnodesConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        AllnodesProvider {
            client: forward_proxy_client,
            supported_chains,
            api_key: provider_config.api_key.clone(),
        }
    }
}

impl RpcProviderFactory<AllnodesConfig> for AllnodesWsProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &AllnodesConfig) -> Self {
        let supported_chains: HashMap<String, String> = provider_config
            .supported_ws_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        AllnodesWsProvider {
            supported_chains,
            api_key: provider_config.api_key.clone(),
        }
    }
}
