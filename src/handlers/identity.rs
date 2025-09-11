use {
    super::{proxy::rpc_call, RpcQueryParams, SdkInfoParams},
    crate::{
        analytics::IdentityLookupInfo,
        database::helpers::get_names_by_address,
        error::RpcError,
        json_rpc::{JsonRpcError, JsonRpcResponse},
        state::AppState,
        utils::{crypto, network},
    },
    async_trait::async_trait,
    axum::{
        body::to_bytes,
        extract::{ConnectInfo, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    chrono::{DateTime, TimeDelta, Utc},
    core::fmt,
    ethers::{
        abi::Address,
        providers::{JsonRpcClient, Middleware, Provider, ProviderError},
        types::H160,
        utils::to_checksum,
    },
    hyper::{header::CACHE_CONTROL, HeaderMap, StatusCode},
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::{
        net::SocketAddr,
        sync::Arc,
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    tap::TapFallible,
    tracing::{debug, error, warn},
    wc::metrics::{self, enum_ordinalize::Ordinalize, future_metrics, Enum, FutureExt},
};

const CACHE_TTL: u64 = 60 * 60 * 24;
const CACHE_TTL_DELTA: TimeDelta = TimeDelta::seconds(CACHE_TTL as i64);
const CACHE_TTL_STD: Duration = Duration::from_secs(CACHE_TTL);

const SELF_PROVIDER_ERROR_PREFIX: &str = "SelfProviderError: ";
const EMPTY_RPC_RESPONSE: &str = "0x";
pub const ETHEREUM_MAINNET: &str = "eip155:1";
pub const SOLANA_MAINNET: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";

/// Cap to 150 Kb max size for the identity response
const IDENTITY_RESPONSE_MAX_BYTES: usize = 150 * 1024;

/// Error codes that reflect an `execution reverted` and should proceed with Ok() during
/// the identity avatar lookup because of an absence of the ERC-721 contract address or
/// token ID in the ENS avatar record.
const JSON_RPC_OK_ERROR_CODES: [&str; 4] = ["-32000", "-32003", "-32015", "3"];

/// Check if the provided address string is a valid Solana address
fn is_solana_address(address: &str) -> bool {
    crypto::is_address_valid(address, &crypto::CaipNamespaces::Solana)
}

/// Build an empty identity response and corresponding cache-control header
/// to be used for non-EVM addresses like Solana.
fn build_empty_identity_response_with_cache() -> (IdentityResponse, String) {
    let res = IdentityResponse {
        name: None,
        avatar: None,
        resolved_at: Some(Utc::now()),
    };
    // Cache control for 1 hour
    let ttl_secs = 60 * 60;
    let cache_control = format!("public, max-age={ttl_secs}, s-maxage={ttl_secs}");
    (res, cache_control)
}

/// Record analytics for identity lookups including Solana empty response case.
#[allow(clippy::too_many_arguments)]
fn record_identity_lookup_analytics(
    state: &AppState,
    query: &IdentityQueryParams,
    headers: &HeaderMap,
    client_ip: SocketAddr,
    source: IdentityLookupSource,
    address_evm: Option<H160>,
    address_str: &str,
    name_present: bool,
    avatar_present: bool,
    latency: Duration,
    chain_id_override: Option<&str>,
) {
    let origin = headers
        .get("origin")
        .map(|v| v.to_str().unwrap_or("invalid_header").to_string());

    let (country, continent, region) = state
        .analytics
        .lookup_geo_data(network::get_forwarded_ip(headers).unwrap_or(client_ip.ip()))
        .map(|geo| (geo.country, geo.continent, geo.region))
        .unwrap_or((None, None, None));

    if let Some(address) = address_evm {
        state.analytics.identity_lookup(IdentityLookupInfo::new(
            query,
            address,
            name_present,
            avatar_present,
            source,
            latency,
            origin,
            region,
            country,
            continent,
            query.client_id.clone(),
            query.sender.clone(),
            query.sdk_info.sv.clone(),
            query.sdk_info.st.clone(),
        ));
    } else {
        // Manually construct analytics payload for non-EVM addresses (e.g., Solana)
        let event = IdentityLookupInfo {
            timestamp: wc::analytics::time::now(),
            address_hash: sha256::digest(address_str),
            address: address_str.to_string(),
            name_present,
            avatar_present,
            source: source.as_str().to_string(),
            latency_secs: latency.as_secs_f64(),
            project_id: query.project_id.clone(),
            chain_id: chain_id_override.unwrap_or(SOLANA_MAINNET).to_string(),
            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,
            client_id: query.client_id.clone(),
            sender: query.sender.clone(),
            sv: query.sdk_info.sv.clone(),
            st: query.sdk_info.st.clone(),
        };
        state.analytics.identity_lookup(event);
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityQueryParams {
    pub project_id: String,
    /// Optional flag to control the cache to fetch the data from the provider
    /// or serve from the cache where applicable
    pub use_cache: Option<bool>,
    /// Client ID for analytics
    pub client_id: Option<String>,
    /// Request sender address for analytics
    pub sender: Option<String>,
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdentityResponse {
    name: Option<String>,
    avatar: Option<String>,
    // Preferred saving the resolved_at time instead of relying on Redis cache TTL because
    // getting the current TTL requires a second command & round trip to Redis
    // Optional to support DB migration, can switch to required in the future
    resolved_at: Option<DateTime<Utc>>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<IdentityQueryParams>,
    headers: HeaderMap,
    address: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, headers, address)
        .with_metrics(future_metrics!("handler_task", "name" => "identity"))
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

    // If the address is a valid Solana address, build an empty identity response
    // and return early. This function can also be used after emitting analytics.
    if is_solana_address(&address) {
        let (res, cache_control) = build_empty_identity_response_with_cache();
        // Record analytics for Solana address with empty response
        record_identity_lookup_analytics(
            &state,
            &query.0,
            &headers,
            connect_info.0,
            IdentityLookupSource::Local,
            None,
            &address,
            false,
            false,
            Duration::from_secs(0),
            Some(SOLANA_MAINNET),
        );
        return Ok(([(CACHE_CONTROL, cache_control)], Json(res)).into_response());
    }

    let address = address
        .parse::<Address>()
        .map_err(|_| RpcError::InvalidAddress)?;
    let start = SystemTime::now();
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

    record_identity_lookup_analytics(
        &state,
        &query.0,
        &headers,
        connect_info.0,
        source,
        Some(address),
        "",
        name_present,
        avatar_present,
        latency,
        None,
    );

    let now = Utc::now();
    let ttl_secs = res.resolved_at
        .map(|resolved_at| ttl_from_resolved_at(resolved_at, now))
        // Only happens during initial rollout when `resolved_at` is None, so we don't need to go overboard on the cache
        .unwrap_or(TimeDelta::hours(1))
        .num_seconds();
    let cache_control = format!("public, max-age={ttl_secs}, s-maxage={ttl_secs}");

    Ok(([(CACHE_CONTROL, cache_control)], Json(res)).into_response())
}

fn ttl_from_resolved_at(resolved_at: DateTime<Utc>, now: DateTime<Utc>) -> TimeDelta {
    let expires = resolved_at + CACHE_TTL_DELTA;
    (expires - now).max(TimeDelta::zero())
}

#[derive(Serialize, Clone, Copy, Debug, Ordinalize)]
pub enum IdentityLookupSource {
    /// Redis cached results
    Cache,
    /// ENS contract name resolution
    Rpc,
    /// Local name resolver
    Local,
}

impl metrics::Enum for IdentityLookupSource {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Cache => "cache",
            Self::Rpc => "rpc",
            Self::Local => "local",
        }
    }
}

#[tracing::instrument(skip_all, level = "debug")]
async fn lookup_identity(
    address: H160,
    State(state): State<Arc<AppState>>,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    Query(query): Query<IdentityQueryParams>,
    headers: HeaderMap,
) -> Result<(IdentityLookupSource, IdentityResponse), RpcError> {
    let address_with_checksum = to_checksum(&address, None);
    let cache_record_key = format!("{address_with_checksum}-v1");

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
            let value = cache.get(&cache_record_key).await?;
            state.metrics.add_identity_lookup_cache_latency(cache_start);
            if let Some(response) = value {
                return Ok((IdentityLookupSource::Cache, response));
            }
        }
    }

    // Lookup for the name in ENS first
    let mut resolved_by = IdentityLookupSource::Rpc;
    let mut res = lookup_identity_rpc(
        address,
        state.clone(),
        connect_info,
        query.project_id,
        headers,
        query.sdk_info,
    )
    .await?;

    // Lookup for the name in local name resolver if no ENS found
    if res.name.is_none() {
        match get_names_by_address(address_with_checksum.clone(), &state.postgres).await {
            Ok(names) => {
                // Our API v1 support only one name per address, using the first name
                if let Some(name_first) = names.first() {
                    let avatar = name_first
                        .attributes
                        .as_ref()
                        .and_then(|attributes| attributes.get("avatar"))
                        .map(|v| v.to_string());

                    resolved_by = IdentityLookupSource::Local;
                    res.name = Some(name_first.name.clone());
                    res.avatar = avatar;
                }
            }
            Err(e) => {
                error!("Error on local name resolution: {}", e);
                return Err(RpcError::InternalNameResolverError);
            }
        }
    }

    if enable_cache {
        if let Some(cache) = &state.identity_cache {
            debug!("Saving to cache");
            let cache = cache.clone();
            let res = res.clone();
            // Do not block on cache write.
            tokio::spawn(async move {
                let cache_start = SystemTime::now();
                cache
                    .set(&cache_record_key, &res, Some(CACHE_TTL_STD))
                    .await
                    .tap_err(|err| {
                        warn!(
                            "failed to cache identity lookup (cache_key:{cache_record_key}): \
                             {err:?}"
                        )
                    })
                    .ok();
                state.metrics.add_identity_lookup_cache_latency(cache_start);
                debug!("Setting cache success");
            });
        }
    }

    Ok((resolved_by, res))
}

#[tracing::instrument(skip_all, level = "debug")]
async fn lookup_identity_rpc(
    address: H160,
    state: Arc<AppState>,
    connect_info: SocketAddr,
    project_id: String,
    headers: HeaderMap,
    sdk_info: SdkInfoParams,
) -> Result<IdentityResponse, RpcError> {
    let provider = Provider::new(SelfProvider {
        state: state.clone(),
        connect_info,
        query: RpcQueryParams {
            project_id,
            // ENS registry contract is only deployed on mainnet
            chain_id: ETHEREUM_MAINNET.to_owned(),
            provider_id: None,
            session_id: None,
            source: Some(crate::analytics::MessageSource::Identity),
            sdk_info,
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

    Ok(IdentityResponse {
        name,
        avatar,
        resolved_at: Some(Utc::now()),
    })
}

#[tracing::instrument(level = "debug")]
pub fn handle_rpc_error(error: ProviderError) -> Result<(), RpcError> {
    match error {
        ProviderError::CustomError(e) if e.starts_with(SELF_PROVIDER_ERROR_PREFIX) => {
            let error_detail = e.trim_start_matches(SELF_PROVIDER_ERROR_PREFIX);
            // Exception for no available JSON-RPC providers
            if error_detail.contains("503 Service Unavailable")
                || error_detail.contains("400 Bad Request")
            {
                return Err(RpcError::IdentityProviderError(
                    "No available JSON-RPC providers".into(),
                ));
            }
            // Proceed with Ok() if the error is related to the contract call error
            // since there should be a wrong NFT avatar contract address.
            if error_detail.contains("Contract call error") {
                debug!(
                    "Contract call error while looking up identity: {:?}",
                    error_detail
                );
                return Ok(());
            }
            // Check if the error is GenericParameterError which means that the
            // node returned null malformed response
            if error_detail.contains("Generic parameter error") {
                return Err(RpcError::IdentityProviderError(
                    "Malformed response from the JSON-RPC provider on ENS name resolution".into(),
                ));
            }
            // Check the list of error codes that reflects an execution reverted
            // and should proceed with Ok()
            for &code in &JSON_RPC_OK_ERROR_CODES {
                if error_detail.contains(&format!("code: {code},")) {
                    debug!(
                        "JsonRpcError code {} while looking up identity: {:?}",
                        code, error_detail
                    );
                    return Ok(());
                }
            }

            Err(RpcError::IdentityLookup(error_detail.to_string()))
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

#[tracing::instrument(skip(provider), level = "debug")]
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

    #[error("Contract call error: {0}")]
    ContractCallError(String),
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
        ProviderError::CustomError(format!("{SELF_PROVIDER_ERROR_PREFIX}{value}"))
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
            .as_secs();

        let response = rpc_call(
            self.state.clone(),
            self.connect_info,
            self.query.clone(),
            self.headers.clone(),
            serde_json::to_vec(&crypto::JsonRpcRequest {
                id: id.into(),
                jsonrpc: crypto::JSON_RPC_VERSION.clone(),
                method: method.to_owned().into(),
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
        let bytes = to_bytes(response.into_body(), IDENTITY_RESPONSE_MAX_BYTES)
            .await
            .map_err(SelfProviderError::ProviderBody)?;

        let response = serde_json::from_slice::<JsonRpcResponse>(&bytes)
            .map_err(SelfProviderError::ProviderBodySerde)?;

        let result = match response {
            JsonRpcResponse::Error(e) => return Err(SelfProviderError::JsonRpcError(e)),
            JsonRpcResponse::Result(r) => {
                // We shouldn't process with `0x` result because this leads to the ethers-rs
                // panic when looking for an avatar. This is a workaround for the ethers-rs
                // when avatar pointing to the wrong ERC-721 contract address.
                if r.result == EMPTY_RPC_RESPONSE {
                    return Err(SelfProviderError::ContractCallError(
                        "Empty response from the contract call".into(),
                    ));
                } else {
                    r.result
                }
            }
        };
        let result = serde_json::from_value(result).map_err(|e| {
            SelfProviderError::GenericParameterError(format!(
                "Caller should always provide generic parameter R=Bytes: {e}"
            ))
        })?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn full_ttl_when_resolved_now() {
        let now = Utc::now();
        assert_eq!(ttl_from_resolved_at(now, now), CACHE_TTL_DELTA);
    }

    #[test]
    fn expires_now() {
        let now = Utc::now();
        assert_eq!(
            ttl_from_resolved_at(now - CACHE_TTL_DELTA, now),
            TimeDelta::zero()
        );
    }

    #[test]
    fn expires_past() {
        let now = Utc::now();
        assert_eq!(
            ttl_from_resolved_at(now - CACHE_TTL_DELTA - TimeDelta::days(1), now),
            TimeDelta::zero()
        );
    }

    #[test]
    fn deserialize_identity_response_with_no_resolved_at() {
        serde_json::from_value::<IdentityResponse>(json!({
            "name": "name",
            "avatar": "avatar"
        }))
        .unwrap();
    }
}
