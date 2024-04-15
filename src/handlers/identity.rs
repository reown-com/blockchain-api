use {
    super::{proxy::rpc_call, RpcQueryParams, HANDLER_TASK_METRICS},
    crate::{
        analytics::IdentityLookupInfo,
        error::RpcError,
        json_rpc::{JsonRpcError, JsonRpcResponse},
        state::AppState,
        utils::{crypto, network},
    },
    async_trait::async_trait,
    axum::{
        extract::{ConnectInfo, Path, Query, State},
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

const SELF_PROVIDER_ERROR_PREFIX: &str = "SelfProviderError: ";
const EMPTY_RPC_RESPONSE: &str = "0x";
pub const ETHEREUM_MAINNET: &str = "eip155:1";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityResponse {
    name: Option<String>,
    avatar: Option<String>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<IdentityQueryParams>,
    headers: HeaderMap,
    address: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, headers, address)
        .with_metrics(HANDLER_TASK_METRICS.with_name("identity"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<IdentityQueryParams>,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query.project_id)
        .await?;

    let start = SystemTime::now();
    let address = address
        .parse::<Address>()
        .map_err(|_| RpcError::InvalidAddress)?;

    let identity_result = lookup_identity(
        address,
        state.clone(),
        connect_info,
        query.clone(),
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
            .lookup_geo_data(
                network::get_forwarded_ip(headers).unwrap_or_else(|| connect_info.0.ip()),
            )
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityQueryParams {
    pub project_id: String,
    /// Optional flag to control the cache to fetch the data from the provider
    /// or serve from the cache where applicable
    pub use_cache: Option<bool>,
}

#[tracing::instrument(skip_all, level = "debug")]
async fn lookup_identity(
    address: H160,
    State(state): State<Arc<AppState>>,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Query(query): Query<IdentityQueryParams>,
    headers: HeaderMap,
) -> Result<(IdentityLookupSource, IdentityResponse), RpcError> {
    let cache_key = format!("{}", address);

    // Check if we should enable cache control for allow listed Project ID
    // The cache is enabled by default
    let enable_cache = if let Some(use_cache) = query.use_cache {
        if let Some(ref testing_project_id) = state.config.server.testing_project_id {
            if crypto::constant_time_eq(testing_project_id, &query.project_id) {
                use_cache
            } else {
                return Err(RpcError::InvalidParameter(format!(
                    "The project ID {} is not allowed to use `use_cache` parameter",
                    query.project_id
                )));
            }
        } else {
            return Err(RpcError::InvalidParameter(
                "Use of `use_cache` parameter is disabled".into(),
            ));
        }
    } else {
        true
    };

    if enable_cache {
        if let Some(cache) = &state.identity_cache {
            debug!("Checking cache for identity");
            let cache_start = SystemTime::now();
            let value = cache.get(&cache_key).await?;
            state.metrics.add_identity_lookup_cache_latency(cache_start);
            if let Some(response) = value {
                return Ok((IdentityLookupSource::Cache, response));
            }
        }
    }

    let res = lookup_identity_rpc(
        address,
        state.clone(),
        connect_info,
        query.project_id,
        headers,
    )
    .await?;

    if enable_cache {
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
    }

    Ok((IdentityLookupSource::Rpc, res))
}

#[tracing::instrument(skip_all, level = "debug")]
async fn lookup_identity_rpc(
    address: H160,
    state: Arc<AppState>,
    connect_info: SocketAddr,
    project_id: String,
    headers: HeaderMap,
) -> Result<IdentityResponse, RpcError> {
    let provider = Provider::new(SelfProvider {
        state: state.clone(),
        connect_info,
        query: RpcQueryParams {
            project_id,
            // ENS registry contract is only deployed on mainnet
            chain_id: ETHEREUM_MAINNET.to_owned(),
            provider_id: None,
        },
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

#[tracing::instrument(level = "debug")]
pub fn handle_rpc_error(error: ProviderError) -> Result<(), RpcError> {
    match error {
        ProviderError::CustomError(e) if e.starts_with(SELF_PROVIDER_ERROR_PREFIX) => {
            let error_detail = e.trim_start_matches(SELF_PROVIDER_ERROR_PREFIX);
            // Exceptions for the detailed HTTP error return on RPC call
            if error_detail.contains("503 Service Unavailable") {
                Err(RpcError::ProviderError)
            } else {
                Err(RpcError::IdentityLookup(error_detail.to_string()))
            }
        }
        ProviderError::CustomError(e) => {
            debug!("Custom error while looking up identity: {:?}", e);
            Ok(())
        }
        _ => {
            debug!(
                "Non-matching provider error while looking up identity: {:?}",
                error
            );
            Ok(())
        }
    }
}

#[tracing::instrument(skip_all, level = "debug")]
async fn lookup_name(
    provider: &Provider<SelfProvider>,
    address: Address,
) -> Result<Option<String>, RpcError> {
    provider.lookup_address(address).await.map_or_else(
        |error| match handle_rpc_error(error) {
            Ok(_) => Ok(None),
            Err(e) => Err(e),
        },
        |name| Ok(Some(name)),
    )
}

#[tracing::instrument(skip(provider))]
async fn lookup_avatar(
    provider: &Provider<SelfProvider>,
    name: &str,
) -> Result<Option<String>, RpcError> {
    provider
        .resolve_avatar(name)
        .await
        .map(|url| url.to_string())
        .map_or_else(
            |error| match handle_rpc_error(error) {
                Ok(_) => Ok(None),
                Err(e) => Err(e),
            },
            |avatar| Ok(Some(avatar)),
        )
}

struct SelfProvider {
    state: Arc<AppState>,
    connect_info: SocketAddr,
    query: RpcQueryParams,
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

    #[error("Generic parameter error: {0}")]
    GenericParameterError(String),
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

        let response = rpc_call(
            self.state.clone(),
            self.connect_info,
            self.query.clone(),
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
            JsonRpcResponse::Result(r) => {
                // We shouldn't process with `0x` result because this leads to the ethers-rs
                // panic when looking for an avatar
                if r.result == EMPTY_RPC_RESPONSE {
                    return Err(SelfProviderError::ProviderError {
                        status: StatusCode::METHOD_NOT_ALLOWED,
                        body: format!("JSON-RPC result is {}", EMPTY_RPC_RESPONSE),
                    });
                } else {
                    r.result
                }
            }
        };
        let result = serde_json::from_value(result).map_err(|_| {
            SelfProviderError::GenericParameterError(
                "Caller always provides generic parameter R=Bytes".into(),
            )
        })?;
        Ok(result)
    }
}
