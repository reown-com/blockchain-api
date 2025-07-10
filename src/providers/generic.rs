use {
    super::{
        Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory, RpcQueryParams,
        RpcWsProvider, WS_PROXY_TASK_METRICS,
    },
    crate::{
        env::{GenericConfig, ProviderConfig},
        error::{RpcError, RpcResult},
        providers::{
            is_internal_error_rpc_code, is_node_error_rpc_message,
            is_rate_limited_error_rpc_message,
        },
        ws,
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    axum_tungstenite::WebSocketUpgrade,
    hyper::{client::HttpConnector, http, Client, Method},
    hyper_tls::HttpsConnector,
    tracing::debug,
    wc::future::FutureExt,
};

#[derive(Debug)]
pub struct GenericProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
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
                .map_err(|e| RpcError::AxumTungstenite(Box::new(e)))?;

        Ok(ws.on_upgrade(move |socket| {
            ws::proxy(query_params.project_id, socket, websocket_provider)
                .with_metrics(WS_PROXY_TASK_METRICS.with_name("generic"))
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
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(self.config.provider.url.clone())
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await?;

        if let Ok(json_response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if let Some(error) = &json_response.error {
                debug!(
                    "Strange: provider returned JSON RPC error, but status {status} is success: \
                 Generic: {json_response:?}"
                );
                if is_internal_error_rpc_code(error.code) {
                    if is_rate_limited_error_rpc_message(&error.message) {
                        return Ok((http::StatusCode::TOO_MANY_REQUESTS, body).into_response());
                    }
                    if is_node_error_rpc_message(&error.message) {
                        return Ok((http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response());
                    }
                }
            }
        }

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
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

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
