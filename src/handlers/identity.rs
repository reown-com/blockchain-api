use {
    super::HANDLER_TASK_METRICS,
    crate::{
        analytics::IdentityLookupInfo,
        error::RpcError,
        handlers::RpcQueryParams,
        json_rpc::{JsonRpcError, JsonRpcResponse},
        project::ProjectDataError,
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
    hyper::{body::to_bytes, HeaderMap, StatusCode},
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

    let identity_result = lookup_identity(
        address,
        state.clone(),
        connect_info,
        query.clone(),
        path,
        headers.clone(),
    )
    .await;

    state.metrics.add_identity_lookup();
    let (source, res) = identity_result?;
    state.metrics.add_identity_lookup_success(&source);
    let latency = start.elapsed().unwrap_or(Duration::from_secs(0));
    state.metrics.add_identity_lookup_latency(latency, &source);

    let name_present = res.name.is_some();
    if name_present {
        state.metrics.add_identity_lookup_name_present();
    }
    let avatar_present = res.avatar.is_some();
    if avatar_present {
        state.metrics.add_identity_lookup_avatar_present();
    }

    {
        let origin = headers
            .get("origin")
            .map(|v| v.to_str().unwrap_or("invalid_header").to_string());

        let (country, continent, region) = state
            .analytics
            .lookup_geo_data(connect_info.0.ip())
            .map(|geo| (geo.country, geo.continent, geo.region))
            .unwrap_or((None, None, None));

        state.analytics.identity_lookup(IdentityLookupInfo::new(
            &query.0,
            address,
            name_present,
            avatar_present,
            source,
            latency,
            origin,
            region,
            country,
            continent,
        ));
    }

    Ok(Json(res).into_response())
}

#[derive(Serialize, Clone)]
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
        let name_result = lookup_name(&provider, address).await;

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
        let avatar_result = lookup_avatar(&provider, name).await;

        state.metrics.add_identity_lookup_avatar();
        let avatar = avatar_result?;
        state.metrics.add_identity_lookup_avatar_success();
        state
            .metrics
            .add_identity_lookup_avatar_latency(avatar_lookup_start);

        avatar
    } else {
        None
    };

    Ok(IdentityResponse { name, avatar })
}

const SELF_PROVIDER_ERROR_PREFIX: &str = "SelfProviderError: ";

async fn lookup_name(
    provider: &Provider<SelfProvider>,
    address: Address,
) -> Result<Option<String>, RpcError> {
    provider.lookup_address(address).await.map_or_else(
        |e| match e {
            ProviderError::CustomError(e)
                if &e == "SelfProviderError: RpcError: ProjectDataError(NotFound)" =>
            {
                Err(RpcError::ProjectDataError(ProjectDataError::NotFound))
            }
            ProviderError::CustomError(e) if e.starts_with(SELF_PROVIDER_ERROR_PREFIX) => Err(
                RpcError::NameLookup(e[SELF_PROVIDER_ERROR_PREFIX.len()..].to_string()),
            ),
            e => {
                debug!("Error while looking up name: {e:?}");
                Ok(None)
            }
        },
        |name| Ok(Some(name)),
    )
}

async fn lookup_avatar(
    provider: &Provider<SelfProvider>,
    name: &str,
) -> Result<Option<String>, RpcError> {
    provider
        .resolve_avatar(name)
        .await
        .map(|url| url.to_string())
        .map_or_else(
            |e| match e {
                ProviderError::CustomError(e)
                    if &e == "SelfProviderError: RpcError: ProjectDataError(NotFound)" =>
                {
                    Err(RpcError::ProjectDataError(ProjectDataError::NotFound))
                }
                ProviderError::CustomError(e) if e.starts_with(SELF_PROVIDER_ERROR_PREFIX) => Err(
                    RpcError::AvatarLookup(e[SELF_PROVIDER_ERROR_PREFIX.len()..].to_string()),
                ),
                e => {
                    debug!("Error while looking up avatar: {e:?}");
                    Ok(None)
                }
            },
            |avatar| Ok(Some(avatar)),
        )
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
        ProviderError::CustomError(format!("{}{}", SELF_PROVIDER_ERROR_PREFIX, value))
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
