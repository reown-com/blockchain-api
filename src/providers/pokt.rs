use {
    super::{
        is_internal_error_rpc_code, is_node_error_rpc_message, is_rate_limited_error_rpc_message,
        Provider, ProviderKind, RateLimited, RpcProvider, RpcProviderFactory,
    },
    crate::{
        env::PoktConfig,
        error::{RpcError, RpcResult},
    },
    async_trait::async_trait,
    axum::{
        http::HeaderValue,
        response::{IntoResponse, Response},
    },
    hyper::{self, client::HttpConnector, Client, Method, StatusCode},
    hyper_tls::HttpsConnector,
    serde::Deserialize,
    std::collections::HashMap,
    tracing::debug,
};

#[derive(Debug)]
pub struct PoktProvider {
    pub client: Client<HttpsConnector<HttpConnector>>,
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
    async fn proxy(&self, chain_id: &str, body: hyper::body::Bytes) -> RpcResult<Response> {
        let chain = &self
            .supported_chains
            .get(chain_id)
            .ok_or(RpcError::ChainNotFound)?;

        let uri = format!("https://{}.rpc.grove.city/v1/{}", chain, self.project_id);

        let hyper_request = hyper::http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(hyper::body::Body::from(body))?;

        let response = self.client.request(hyper_request).await?;
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await?;

        if status.is_success() {
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
                        // Handle other internal error codes with message-based classification
                        code if is_internal_error_rpc_code(code) => {
                            if is_rate_limited_error_rpc_message(&error.message) {
                                return Ok((StatusCode::TOO_MANY_REQUESTS, body).into_response());
                            }
                            if is_node_error_rpc_message(&error.message) {
                                return Ok(
                                    (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // As an internal RPC node error the InternalErrorResponse is used for the response
        // that should be considered as an HTTP 5xx error from the provider
        if let Ok(response) = serde_json::from_slice::<InternalErrorResponse>(&body) {
            let error = response.error;
            let request_id = response.request_id;
            if error.contains("try again later") {
                return Ok((StatusCode::SERVICE_UNAVAILABLE, body).into_response());
            } else {
                debug!(
                    "Pokt provider returned JSON RPC success status, but got the \
                    error response structure with the following error: {error} \
                    the request_id is {request_id}",
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
        let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let supported_chains: HashMap<String, String> = provider_config
            .supported_chains
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();

        PoktProvider {
            client: forward_proxy_client,
            supported_chains,
            project_id: provider_config.project_id.clone(),
        }
    }
}
