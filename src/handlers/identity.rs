use {
    crate::{
        error::RpcError,
        extractors::method::Method,
        handlers::RpcQueryParams,
        json_rpc::{JsonRpcError, JsonRpcResponse},
        state::AppState,
    },
    async_trait::async_trait,
    axum::{
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        Json,
    },
    core::fmt,
    ethers::{
        abi::Address,
        providers::{JsonRpcClient, Middleware, Provider, ProviderError},
    },
    hyper::{body::to_bytes, HeaderMap, Method as HyperMethod, StatusCode},
    serde::{de::DeserializeOwned, Serialize},
    std::{
        net::SocketAddr,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    },
    tracing::debug,
};

#[derive(Serialize)]
pub struct IdentityResponse {
    name: String,
    avatar: Option<String>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<RpcQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Json<IdentityResponse>, RpcError> {
    let start = SystemTime::now();
    state.metrics.add_identity_lookup();

    let address = address
        .parse::<Address>()
        .map_err(|_| RpcError::IdentityInvalidAddress)?;

    let provider = Provider::new(SelfProvider {
        state: state.clone(),
        connect_info,
        query,
        path,
        headers,
    });

    let name_lookup_start = SystemTime::now();
    let name = provider
        .lookup_address(address)
        .await
        .map_err(|e| match e {
            ProviderError::EnsError(e) | ProviderError::EnsNotOwned(e) => {
                RpcError::IdentityNotFound(e)
            }
            e => RpcError::EthersProviderError(e),
        })?;
    let tld = name
        .rsplit('.')
        .next()
        .expect("split always returns at least 1 item, even if splitting empty string");
    state
        .metrics
        .add_identity_lookup_name_duration(name_lookup_start, tld.to_string());
    state
        .metrics
        .add_identity_lookup_name_success(tld.to_string());

    let avatar_lookup_start = SystemTime::now();
    let avatar = provider
        .resolve_avatar(&name)
        .await
        .map_or_else(
            |e| match e {
                ProviderError::EnsError(_) | ProviderError::EnsNotOwned(_) => Ok(None),
                ProviderError::CustomError(e) if e.starts_with("relative URL without a base") => {
                    // Seems not having an `avatar` field returns this error
                    Ok(None)
                }
                e => Err(RpcError::EthersProviderError(e)),
            },
            |url| Ok(Some(url)),
        )?
        .map(|url| url.to_string());
    state
        .metrics
        .add_identity_lookup_avatar_duration(avatar_lookup_start, tld.to_string());
    state
        .metrics
        .add_identity_lookup_avatar_success(tld.to_string());
    if avatar.is_some() {
        state
            .metrics
            .add_identity_lookup_avatar_present(tld.to_string());
    }

    state
        .metrics
        .add_identity_lookup_latency(start, tld.to_string());
    state.metrics.add_identity_lookup_success(tld.to_string());

    let res = IdentityResponse { name, avatar };

    Ok(Json(res))
}

struct SelfProvider {
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<RpcQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
}

impl fmt::Debug for SelfProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelfProvider").finish()
    }
}

#[derive(Serialize)]
struct JsonRpcRequest<T: Serialize + Send + Sync> {
    id: String,
    jsonrpc: String,
    method: String,
    params: T,
}

#[derive(Debug, thiserror::Error)]
pub enum SelfProviderError {
    #[error(transparent)]
    RpcError(#[from] Box<RpcError>),

    #[error("proxy_handler status code not OK: {status} {body}")]
    ProviderError { status: StatusCode, body: String },

    #[error("problem with getting provider body: {0}")]
    ProviderBody(#[from] axum::Error),

    #[error("problem with deserializing provider body: {0}")]
    ProviderBodySerde(#[from] serde_json::Error),

    #[error("JsonRpcError: {0:?}")]
    JsonRpcError(JsonRpcError),
}

impl ethers::providers::RpcError for SelfProviderError {
    fn as_error_response(&self) -> Option<&ethers::providers::JsonRpcError> {
        None
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        if let Self::ProviderBodySerde(e) = self {
            Some(e)
        } else {
            None
        }
    }
}

impl From<SelfProviderError> for ProviderError {
    fn from(value: SelfProviderError) -> Self {
        ProviderError::CustomError(format!("{}", value))
    }
}

#[async_trait]
impl JsonRpcClient for SelfProvider {
    type Error = SelfProviderError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, Self::Error> {
        debug!("Got SelfProvider request");

        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("can get current time")
            .as_millis()
            .to_string();

        let response = super::proxy::handler(
            self.state.clone(),
            self.connect_info,
            Query(RpcQueryParams {
                chain_id: self.query.chain_id.clone(),
                project_id: self.query.project_id.clone(),
            }),
            Method(HyperMethod::POST),
            self.path.clone(),
            self.headers.clone(),
            serde_json::to_vec(&JsonRpcRequest {
                id,
                jsonrpc: "2.0".to_string(),
                method: method.to_owned(),
                params,
            })
            .expect("Should be able to serialize a JsonRpcRequest")
            .into(),
        )
        .await
        .map_err(|e| SelfProviderError::RpcError(Box::new(e)))?;

        if response.status() != StatusCode::OK {
            return Err(SelfProviderError::ProviderError {
                status: response.status(),
                body: format!("{:?}", response.body()),
            });
        }

        let bytes = to_bytes(response.into_body())
            .await
            .map_err(SelfProviderError::ProviderBody)?;

        let response = serde_json::from_slice::<JsonRpcResponse>(&bytes)
            .map_err(SelfProviderError::ProviderBodySerde)?;

        match response {
            JsonRpcResponse::Error(e) => return Err(SelfProviderError::JsonRpcError(e)),
            JsonRpcResponse::Result(r) => Ok(serde_json::from_value(r.result)
                .expect("Caller should provide generic parameter of type Bytes")),
        }
    }
}
