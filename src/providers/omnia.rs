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

#[derive(Clone)]
pub struct OmniatechProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
    pub supported_chains: HashMap<String, (String, Weight)>,
}

impl Provider for OmniatechProvider {
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
        ProviderKind::Omniatech
    }
}

#[async_trait]
impl RpcProvider for OmniatechProvider {
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

        let uri = format!("https://endpoints.omniatech.io/v1/{}/mainnet/public", chain);

        let hyper_request = hyper::http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;

        let (body_bytes, response) = copy_body_bytes(response).await.unwrap();

        if is_rate_limited(body_bytes) {
            return Err(RpcError::Throttled);
        }

        Ok(response.into_response())
    }
}

fn is_rate_limited(body_bytes: Bytes) -> bool {
    let Ok(jsonrpc_response) = serde_json::from_slice::<jsonrpc::Response>(&body_bytes) else {return false};

    if let Some(err) = jsonrpc_response.error {
        // Code used by 1rpc to indicate rate limited request
        // https://docs.ata.network/1rpc/introduction/#limitations
        if err.code == -32001 {
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
