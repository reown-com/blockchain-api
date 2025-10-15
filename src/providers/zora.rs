use {
    super::{
        Provider,
        ProviderKind,
        RateLimited,
        RpcProvider,
        RpcProviderFactory,
        RpcQueryParams,
        RpcWsProvider,
    },
    crate::{
        env::ZoraConfig,
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
    tracing::debug,
    wc::metrics::{future_metrics, FutureExt},
};

#[derive(Debug)]
pub struct ZoraProvider {
    pub client: reqwest::Client,
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
    #[tracing::instrument(skip_all, fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(&query_params.chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let project_id = query_params.project_id;

        let (websocket_provider, _) = async_tungstenite::tokio::connect_async(uri)
            .await
            .map_err(|e| RpcError::WebSocketError(e.to_string()))?;

        Ok(ws.on_upgrade(move |socket| {
            ws::proxy(project_id, socket, websocket_provider)
                .with_metrics(future_metrics!("ws_proxy_task", "name" => "zora"))
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
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let uri = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if response.error.is_some() && status.is_success() {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                     Zora: {response:?}"
                );
            }
        }

        let mut response = (status, body).into_response();
        response
            .headers_mut()
            .insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok(response)
    }
}

impl RpcProviderFactory<ZoraConfig> for ZoraProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &ZoraConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();
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
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &ZoraConfig) -> Self {
        let supported_chains: HashMap<String, String> = provider_config
            .supported_ws_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        ZoraWsProvider { supported_chains }
    }
}
