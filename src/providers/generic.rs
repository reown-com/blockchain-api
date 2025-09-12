use {
    super::{
        Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory, RpcQueryParams,
        RpcWsProvider,
    },
    crate::{
        env::{GenericConfig, ProviderConfig},
        error::{RpcError, RpcResult},
        ws,
    },
    async_trait::async_trait,
    axum::extract::ws::WebSocketUpgrade,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::http,
    wc::metrics::{future_metrics, FutureExt},
};

#[derive(Debug)]
pub struct GenericProvider {
    pub client: reqwest::Client,
    pub config: GenericConfig,
}

#[derive(Debug)]
pub struct GenericWsProvider {
    pub config: GenericConfig,
}

impl Provider for GenericWsProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.config.caip2 == chain_id
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        vec![self.config.caip2.clone()]
    }

    fn provider_kind(&self) -> ProviderKind {
        self.config.provider_kind()
    }
}

#[async_trait]
impl RpcWsProvider for GenericWsProvider {
    #[tracing::instrument(skip_all, fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(
        &self,
        ws: WebSocketUpgrade,
        query_params: RpcQueryParams,
    ) -> RpcResult<Response> {
        let (websocket_provider, _) =
            async_tungstenite::tokio::connect_async(self.config.provider.url.clone())
                .await
                .map_err(|e| RpcError::WebSocketError(e.to_string()))?;

        Ok(ws.on_upgrade(move |socket| {
            ws::proxy(query_params.project_id, socket, websocket_provider)
                .with_metrics(future_metrics!("ws_proxy_task", "name" => "generic"))
        }))
    }
}

#[async_trait]
impl RateLimited for GenericWsProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool
    where
        Self: Sized,
    {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

impl Provider for GenericProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.config.caip2 == chain_id
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        vec![self.config.caip2.clone()]
    }

    fn provider_kind(&self) -> ProviderKind {
        self.config.provider_kind()
    }
}

#[async_trait]
impl RateLimited for GenericProvider {
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == http::StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for GenericProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let response = self
            .client
            .post(self.config.provider.url.clone())
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

impl RpcProviderFactory<GenericConfig> for GenericProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &GenericConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();

        Self {
            client: forward_proxy_client,
            config: provider_config.clone(),
        }
    }
}

impl RpcProviderFactory<GenericConfig> for GenericWsProvider {
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &GenericConfig) -> Self {
        Self {
            config: provider_config.clone(),
        }
    }
}
