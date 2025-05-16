use {
    super::HANDLER_TASK_METRICS,
    crate::{
        error::RpcError,
        providers::SupportedBundlerOps,
        state::AppState,
        utils::{
            crypto::{self, disassemble_caip2},
            simple_request_json::SimpleRequestJson,
        },
    },
    alloy::rpc::json_rpc::Id,
    axum::{
        extract::{Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    tracing::info,
    url::Url,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BundlerQueryParams {
    pub project_id: String,
    pub chain_id: String,
    // matching the name of the query param Universal Provider passes: https://github.com/WalletConnect/walletconnect-monorepo/blob/475f2813b6f0fe0d3dc01eeaee9182e331c56daa/providers/universal-provider/src/providers/eip155.ts#L250
    pub bundler: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BundlerJsonRpcRequest {
    pub id: Id,
    pub jsonrpc: Arc<str>,
    pub method: SupportedBundlerOps,
    pub params: serde_json::Value,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    query_params: Query<BundlerQueryParams>,
    SimpleRequestJson(request_payload): SimpleRequestJson<BundlerJsonRpcRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, query_params, request_payload)
        .with_metrics(HANDLER_TASK_METRICS.with_name("bundler_ops"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    State(state): State<Arc<AppState>>,
    Query(query_params): Query<BundlerQueryParams>,
    request_payload: BundlerJsonRpcRequest,
) -> Result<Response, RpcError> {
    state
        .validate_project_access_and_quota(&query_params.project_id.clone())
        .await?;
    let evm_chain_id = disassemble_caip2(&query_params.chain_id)?.1;
    info!("bundler endpoint bundler: {:?}", query_params.bundler);
    let result = match query_params.bundler {
        None => {
            state
                .providers
                .bundler_ops_provider
                .bundler_rpc_call(
                    &evm_chain_id,
                    request_payload.id,
                    request_payload.jsonrpc,
                    &request_payload.method,
                    request_payload.params,
                )
                .await?
        }
        Some(bundler) if bundler == "pimlico" => {
            state
                .providers
                .bundler_ops_provider
                .bundler_rpc_call(
                    &evm_chain_id,
                    request_payload.id,
                    request_payload.jsonrpc,
                    &request_payload.method,
                    request_payload.params,
                )
                .await?
        }
        Some(unsafe_bundler) => {
            let url = unsafe_bundler
                .parse::<Url>()
                .map_err(RpcError::UnsupportedBundlerNameUrlParseError)?;
            if url.scheme() != "https" {
                return Err(RpcError::UnsupportedBundlerName(format!(
                    "must be https://, got {}",
                    url.scheme()
                )));
            }
            let domain = url.domain();
            if let Some(domain) = domain {
                if domain.ends_with("localhost")
                    || domain.ends_with("local")
                    || domain.ends_with("localhost.")
                    || domain.ends_with("local.")
                {
                    return Err(RpcError::UnsupportedBundlerName(format!(
                        "domain is not supported, got {}",
                        domain
                    )));
                }
            } else {
                return Err(RpcError::UnsupportedBundlerName(
                    "domain is required".to_owned(),
                ));
            }

            let method = match request_payload.method {
                SupportedBundlerOps::EthSendUserOperation => "eth_sendUserOperation".into(),
                SupportedBundlerOps::EthGetUserOperationReceipt => {
                    "eth_getUserOperationReceipt".into()
                }
                SupportedBundlerOps::EthEstimateUserOperationGas => {
                    "eth_estimateUserOperationGas".into()
                }
                SupportedBundlerOps::PmSponsorUserOperation => "pm_sponsorUserOperation".into(),
                SupportedBundlerOps::PmGetPaymasterData => "pm_getPaymasterData".into(),
                SupportedBundlerOps::PmGetPaymasterStubData => "pm_getPaymasterStubData".into(),
                SupportedBundlerOps::PimlicoGetUserOperationGasPrice => {
                    "pimlico_getUserOperationGasPrice".into()
                }
            };

            let jsonrpc_send_userop_request = crypto::JsonRpcRequest {
                id: request_payload.id,
                jsonrpc: request_payload.jsonrpc,
                method,
                params: request_payload.params,
            };

            state
                .http_client
                .post(url)
                .json(&jsonrpc_send_userop_request)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?
        }
    };

    Ok(Json(result).into_response())
}
