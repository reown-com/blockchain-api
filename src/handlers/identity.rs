use {
    super::HANDLER_TASK_METRICS,
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
        response::{IntoResponse, Response},
        Json,
    },
    core::fmt,
    ethers::{
        abi::Address,
        providers::{JsonRpcClient, Middleware, Provider, ProviderError},
        types::H160,
    },
    hyper::{body::to_bytes, HeaderMap, Method as HyperMethod, StatusCode},
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::{
        net::SocketAddr,
        sync::Arc,
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    tap::TapFallible,
    tracing::{debug, warn},
    wc::future::FutureExt,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityResponse {
    name: Option<String>,
    avatar: Option<String>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<RpcQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    address: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, path, headers, address)
        .with_metrics(HANDLER_TASK_METRICS.with_name("identity"))
        .await
}

async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<RpcQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Response, RpcError> {
    let start = SystemTime::now();

    let address = address
        .parse::<Address>()
        .map_err(|_| RpcError::IdentityInvalidAddress)?;

    let identity_result =
        lookup_identity(address, state.clone(), connect_info, query, path, headers).await;

    state.metrics.add_identity_lookup();
    let (source, res) = identity_result?;
    state.metrics.add_identity_lookup_success(&source);
    state.metrics.add_identity_lookup_latency(start, &source);

    if res.name.is_some() {
        state.metrics.add_identity_lookup_name_present();
    }
    if res.avatar.is_some() {
        state.metrics.add_identity_lookup_avatar_present();
    }

    Ok(Json(res).into_response())
}

pub enum IdentityLookupSource {
    Cache,
    Rpc,
}

impl IdentityLookupSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cache => "cache",
            Self::Rpc => "rpc",
        }
    }
}

async fn lookup_identity(
    address: H160,
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<RpcQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
) -> Result<(IdentityLookupSource, IdentityResponse), RpcError> {
    let cache_key = format!("{}", address);
    if let Some(cache) = &state.identity_cache {
        debug!("Checking cache for identity");
        let cache_start = SystemTime::now();
        let value = cache.get(&cache_key).await?;
        state.metrics.add_identity_lookup_cache_latency(cache_start);
        if let Some(response) = value {
            return Ok((IdentityLookupSource::Cache, response));
        }
    }

    let res =
        lookup_identity_rpc(address, state.clone(), connect_info, query, path, headers).await?;

    if let Some(cache) = &state.identity_cache {
        debug!("Saving to cache");
        let cache = cache.clone();
        let res = res.clone();
        let cache_ttl = Duration::from_secs(60 * 60 * 24);
        // Do not block on cache write.
        tokio::spawn(async move {
            cache
                .set(&cache_key, &res, Some(cache_ttl))
                .await
                .tap_err(|err| {
                    warn!("failed to cache identity lookup (cache_key:{cache_key}): {err:?}")
                })
                .ok();
            debug!("Setting cache success");
        });
    }

    Ok((IdentityLookupSource::Rpc, res))
}

async fn lookup_identity_rpc(
    address: H160,
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<RpcQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
) -> Result<IdentityResponse, RpcError> {
    let provider = Provider::new(SelfProvider {
        state: state.clone(),
        connect_info,
        query,
        path,
        headers,
    });

    let name = {
        debug!("Beginning name lookup");
        let name_lookup_start = SystemTime::now();
        let name_result = provider
            .lookup_address(address)
            .await
            .tap_err(|err| debug!("Error while looking up name: {err:?}"))
            .map_or_else(
                |e| match e {
                    ProviderError::EnsError(_) | ProviderError::EnsNotOwned(_) => Ok(None),
                    e => Err(RpcError::EthersProviderError(e)),
                },
                |name| Ok(Some(name)),
            );

        state.metrics.add_identity_lookup_name();
        let name = name_result?;
        state.metrics.add_identity_lookup_name_success();
        state
            .metrics
            .add_identity_lookup_name_latency(name_lookup_start);

        name
    };

    let avatar = if let Some(name) = &name {
        debug!("Beginning avatar lookup");
        let avatar_lookup_start = SystemTime::now();
        let avatar_result = provider
            .resolve_avatar(name)
            .await
            .tap_err(|err| debug!("Error while looking up avatar: {err:?}"))
            .map_or_else(
                |e| match e {
                    ProviderError::EnsError(_) | ProviderError::EnsNotOwned(_) => Ok(None),
                    ProviderError::CustomError(e) if &e == "Unsupported ERC token type" => {
                        // This is a problem with how the avatar was configured by the user.
                        // Originates from here:
                        // - https://github.com/gakonst/ethers-rs/blob/f9c72f222cbf82219101c8772cfa49ba4205ef1d/ethers-providers/src/ext/erc.rs#L34
                        // - https://github.com/gakonst/ethers-rs/blob/f9c72f222cbf82219101c8772cfa49ba4205ef1d/ethers-providers/src/rpc/provider.rs#L818
                        Ok(None)
                    }
                    ProviderError::CustomError(e)
                        if e.starts_with("relative URL without a base") =>
                    {
                        // Seems not having an `avatar` field returns this error
                        Ok(None)
                    }
                    e => Err(RpcError::EthersProviderError(e)),
                },
                |url| Ok(Some(url)),
            );

        state.metrics.add_identity_lookup_avatar();
        let avatar = avatar_result?;
        state.metrics.add_identity_lookup_avatar_success();
        state
            .metrics
            .add_identity_lookup_avatar_latency(avatar_lookup_start);

        avatar.map(|url| url.to_string())
    } else {
        None
    };

    Ok(IdentityResponse { name, avatar })
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
    #[error("RpcError: {0:?}")]
    RpcError(RpcError),

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
            .expect("Time should't go backwards")
            .as_millis()
            .to_string();

        let response = super::proxy::handler(
            self.state.clone(),
            self.connect_info,
            self.query.clone(),
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
        .map_err(SelfProviderError::RpcError)?;

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

        let result = match response {
            JsonRpcResponse::Error(e) => return Err(SelfProviderError::JsonRpcError(e)),
            JsonRpcResponse::Result(r) => r.result,
        };
        let result = serde_json::from_value(result)
            .expect("Caller always provides generic parameter R=Bytes");
        Ok(result)
    }
}
