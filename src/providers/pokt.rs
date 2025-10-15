use {
    super::{Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory},
    crate::{
        env::PoktConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::{
        http::{HeaderValue, StatusCode},
        response::{IntoResponse, Response},
    },
    serde::Deserialize,
    std::collections::HashMap,
    tracing::debug,
};

#[derive(Debug)]
pub struct PoktProvider {
    pub client: reqwest::Client,
    pub project_id: String,
    pub supported_chains: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InternalErrorResponse {
    pub request_id: String,
    pub error: String,
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
    async fn is_rate_limited(&self, response: &mut Response) -> bool {
        response.status() == StatusCode::TOO_MANY_REQUESTS
    }
}

#[async_trait]
impl RpcProvider for PoktProvider {
    #[tracing::instrument(skip(self, body), fields(provider = %self.provider_kind()), level = "debug")]
    async fn proxy(&self, chain_id: &str, body: bytes::Bytes) -> RpcResult<Response> {
        let chain = self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;
        let uri = format!("https://{}.rpc.grove.city/v1/{}", chain, self.project_id);
        let response = self
            .client
            .post(uri)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;

        if status.is_success() || status.is_client_error() {
            if let Ok(response) = serde_json::from_slice::<jsonrpc::Response>(&body) {
                if let Some(error) = &response.error {
                    debug!(
                        "Strange: provider returned JSON RPC error, but status {status} is \
                         success: Pokt: {response:?}"
                    );
                    match error.code {
                        // Pokt-specific rate limit codes
                        -32004 | -32068 => {
                            return Ok((StatusCode::TOO_MANY_REQUESTS, body).into_response())
                        }
                        // Internal server error code
                        -32603 => {
                            return Ok((StatusCode::INTERNAL_SERVER_ERROR, body).into_response())
                        }
                        _ => {}
                    }
                }
            }
        }

        // As an internal RPC node error the InternalErrorResponse is used for the
        // response that should be considered as an HTTP 5xx error from the
        // provider
        if let Ok(response) = serde_json::from_slice::<InternalErrorResponse>(&body) {
            let error = response.error;
            let request_id = response.request_id;
            if error.contains("try again later") {
                return Ok((StatusCode::SERVICE_UNAVAILABLE, body).into_response());
            } else {
                debug!(
                    "Pokt provider returned JSON RPC success status, but got the error response \
                     structure with the following error: {error} the request_id is {request_id}",
                );
                return Ok((StatusCode::INTERNAL_SERVER_ERROR, body).into_response());
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
    #[tracing::instrument(level = "debug")]
    fn new(provider_config: &PoktConfig) -> Self {
        let forward_proxy_client = reqwest::Client::new();
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        PoktProvider {
            client: forward_proxy_client,
            project_id: provider_config.project_id.clone(),
            supported_chains,
        }
    }
}
