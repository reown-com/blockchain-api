use {
    super::{
        Provider,
        ProviderKind,
        RpcProvider,
        RpcQueryParams,
        RpcWsProvider,
        SupportedChain,
        Weight,
    },
    crate::{
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
    pub supported_chains: HashMap<String, (String, Weight)>,
}

#[derive(Debug)]
pub struct InfuraWsProvider {
    pub project_id: String,
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Provider for InfuraWsProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<SupportedChain> {
        self.supported_chains
            .iter()
            .map(|(k, v)| SupportedChain {
                chain_id: k.clone(),
                weight: v.1.clone(),
            })
            .collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Infura
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
            .ok_or(RpcError::ChainNotFound)?
            .0;

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

    fn supported_caip_chains(&self) -> Vec<SupportedChain> {
        self.supported_chains
            .iter()
            .map(|(k, v)| SupportedChain {
                chain_id: k.clone(),
                weight: v.1.clone(),
            })
            .collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Infura
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
            .ok_or(RpcError::ChainNotFound)?
            .0;

        let uri = format!("https://{}.infura.io/v3/{}", chain, self.project_id);

        let hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?.into_response();

        if is_rate_limited(&response) {
            return Err(RpcError::Throttled);
        }

        Ok(response)
    }
}

fn is_rate_limited(response: &Response) -> bool {
    response.status() == http::StatusCode::TOO_MANY_REQUESTS
}
