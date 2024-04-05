use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::PoktConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::StatusCode,
    reqwest::Client,
    std::collections::HashMap,
    tracing::info,
};

#[derive(Debug)]
pub struct PoktProvider {
    pub client: Client,
    pub project_id: String,
    pub supported_chains: HashMap<String, String>,
}

impl Provider for PoktProvider {
    fn supports_caip_chainid(&self, chain_id: &str) -> bool {
        self.supported_chains.contains_key(chain_id)
    }

    fn supported_caip_chains(&self) -> Vec<String> {
        self.supported_chains.keys().cloned().collect()
    }

    fn provider_kind(&self) -> ProviderKind {
        ProviderKind::Pokt
    }
}

#[async_trait]
impl RateLimited for PoktProvider {
    // async fn is_rate_limited(&self, response: &mut Response) -> bool
    // where
    //     Self: Sized,
    // {
    //     let Ok(bytes) = body::to_bytes(response.body_mut()).await else {return
    // false};     let Ok(jsonrpc_response) =
    // serde_json::from_slice::<jsonrpc::Response>(&bytes) else {return false};

    //     if let Some(err) = jsonrpc_response.error {
    //         // Code used by Pokt to indicate rate limited request
    //         // https://github.com/pokt-foundation/portal-api/blob/e06d1e50abfee8533c58768bb9b638c351b87a48/src/controllers/v1.controller.ts
    //         if err.code == -32068 {
    //             return true;
    //         }
    //     }

    //     let body: axum::body::Body =
    // axum::body::Body::wrap_stream(hyper::body::Body::from(bytes));
    //     let body: UnsyncBoxBody<bytes::Bytes, axum_core::Error> =
    // body.boxed_unsync();     let mut_body = response.body_mut();
    //     false
    // }

    // TODO: Implement rate limiting as this is mocked
    async fn is_rate_limited(&self, _response: &mut Response) -> bool {
        false
    }
}

#[async_trait]
impl RpcProvider for PoktProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()))]
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://{}.rpc.grove.city/v1/{}", chain, self.project_id);

        let response = self
            .client
            .post(uri)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let status = response.status();
        let body = response.bytes().await?;

        if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
            if let Some(error) = &response.error {
                if status.is_success() {
                    info!(
                        "Strange: provider returned JSON RPC error, but status {status} is \
                         success: Pokt: {response:?}"
                    );
                }
                if error.code == -32004 {
                    return Ok((StatusCode::TOO_MANY_REQUESTS, body).into_response());
                }
                if error.code == -32603 {
                    return Ok((StatusCode::INTERNAL_SERVER_ERROR, body).into_response());
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

impl RpcProviderFactory<PoktConfig> for PoktProvider {
    #[tracing::instrument]
    fn new(client: Client, provider_config: &PoktConfig) -> Self {
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        PoktProvider {
            client,
            supported_chains,
            project_id: provider_config.project_id.clone(),
        }
    }
}
