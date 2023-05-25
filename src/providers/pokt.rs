use {
    super::{Provider, ProviderKind, RpcProvider, RpcQueryParams, SupportedChain, Weight},
    crate::error::{RpcError, RpcResult},
    async_trait::async_trait,
    axum::response::{IntoResponse, Response},
    hyper::{
        body::{self, Bytes},
        client::HttpConnector,
        Body,
        Client,
    },
    hyper_tls::HttpsConnector,
    std::collections::HashMap,
};

#[derive(Debug)]
pub struct PoktProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub project_id: String,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for PoktProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        // self.supported_chains
        //     .iter()
        //     .map(|(k, v)| SupportedChain {
        //         chain_id: k.clone(),
        //         weight: v.1.clone(),
        //     })
        //     .collect()
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Pokt
    }
}

#[async_trait]
impl RpcProvider for PoktProvider {
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

        let uri = format!(
            "https://{}.gateway.pokt.network/v1/lb/{}",
            chain, self.project_id
        );

        let hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;

        let (body_bytes, response) = copy_body_bytes(response).await.unwrap();

        if is_rate_limited(&body_bytes).await {
            return Err(RpcError::Throttled);
        }

        Ok(response.into_response())
    }
}

async fn is_rate_limited(body_bytes: &Bytes) -> bool {
    let Ok(jsonrpc_response) = serde_json::from_slice::<jsonrpc::Response>(body_bytes) else {return false};

    if let Some(err) = jsonrpc_response.error {
        // Code used by Pokt to indicate rate limited request
        // https://github.com/pokt-foundation/portal-api/blob/e06d1e50abfee8533c58768bb9b638c351b87a48/src/controllers/v1.controller.ts
        if err.code == -32068 {
            return true;
        }
    }
    false
}

async fn copy_body_bytes(
    response: Response<Body>,
) -> Result<(Bytes, Response<Body>), hyper::Error> {
    let (parts, body) = response.into_parts();
    let bytes = body::to_bytes(body).await?;

    let body = Body::from(bytes.clone());
    Ok((bytes, Response::from_parts(parts, body)))
}
