use {
    crate::{
        error::RpcError,
        extractors::method::Method,
        handlers::RpcQueryParams,
        json_rpc::JsonRpcResponse,
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
    avatar: String,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<RpcQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Json<IdentityResponse>, RpcError> {
    let provider = Provider::new(SelfProvider {
        state,
        connect_info,
        query,
        path,
        headers,
    });

    let name = provider
        .lookup_address(address.parse::<Address>().unwrap())
        .await
        .unwrap();
    let avatar = provider.resolve_avatar(&name).await.unwrap();

    let res = IdentityResponse {
        name,
        avatar: avatar.to_string(),
    };

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

#[async_trait]
impl JsonRpcClient for SelfProvider {
    type Error = ProviderError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, Self::Error> {
        debug!("Got SelfProvider request");

        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
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
            })?
            .into(),
        )
        .await
        .map_err(|e| ProviderError::CustomError(format!("proxy_handler error: {:?}", e)))?;
        if response.status() != StatusCode::OK {
            return Err(ProviderError::CustomError(format!(
                "proxy_handler status code not OK: {} {:?}",
                response.status(),
                response.body()
            )));
        }

        let bytes = to_bytes(response.into_body())
            .await
            .map_err(|e| ProviderError::CustomError(format!("to_bytes error: {:?}", e)))?;

        let response = serde_json::from_slice::<JsonRpcResponse>(&bytes).unwrap();

        match response {
            JsonRpcResponse::Error(e) => {
                return Err(ProviderError::CustomError(format!(
                    "JsonRpcResponse::Error: {:?}",
                    e
                )))
            }
            JsonRpcResponse::Result(r) => Ok(serde_json::from_value(r.result).unwrap()),
        }
    }
}
