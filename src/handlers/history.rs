use {
    super::{SdkInfoParams, ROOTSTOCK_MAINNET_CHAIN_ID, ROOTSTOCK_TESTNET_CHAIN_ID},
    crate::{
        analytics::{HistoryLookupInfo, OnrampHistoryLookupInfo},
        error::RpcError,
        providers::ProviderKind,
        state::AppState,
        utils::{crypto, network},
    },
    axum::{
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, sync::Arc},
    tap::TapFallible,
    tracing::log::{debug, error},
    wc::metrics::{future_metrics, FutureExt},
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQueryParams {
    pub currency: Option<String>,
    pub project_id: String,
    pub chain_id: Option<String>,
    pub cursor: Option<String>,
    pub onramp: Option<String>,
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryResponseBody {
    pub data: Vec<HistoryTransaction>,
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransaction {
    pub id: String,
    pub metadata: HistoryTransactionMetadata,
    pub transfers: Option<Vec<HistoryTransactionTransfer>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransactionMetadata {
    pub operation_type: String,
    pub hash: String,
    pub mined_at: String,
    pub sent_from: String,
    pub sent_to: String,
    pub status: String,
    pub nonce: usize,
    pub application: Option<HistoryTransactionMetadataApplication>,
    pub chain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransactionMetadataApplication {
    pub name: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HistoryTransactionTransfer {
    pub fungible_info: Option<HistoryTransactionFungibleInfo>,
    pub nft_info: Option<HistoryTransactionNFTInfo>,
    pub direction: String,
    pub quantity: HistoryTransactionTransferQuantity,
    pub value: Option<f64>,
    pub price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct HistoryTransactionFungibleInfo {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub icon: Option<HistoryTransactionURLItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct HistoryTransactionURLItem {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct HistoryTransactionTransferQuantity {
    pub numeric: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct HistoryTransactionNFTInfo {
    pub name: Option<String>,
    pub content: Option<HistoryTransactionNFTContent>,
    pub flags: HistoryTransactionNFTInfoFlags,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct HistoryTransactionNFTInfoFlags {
    pub is_spam: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct HistoryTransactionNFTContent {
    pub preview: Option<HistoryTransactionURLandContentTypeItem>,
    pub detail: Option<HistoryTransactionURLandContentTypeItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct HistoryTransactionURLandContentTypeItem {
    pub url: String,
    pub content_type: Option<String>,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<HistoryQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    address: Path<String>,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, path, headers, address)
        .with_metrics(future_metrics!("handler_task", "name" => "transactions"))
        .await
}

#[tracing::instrument(skip_all, level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<HistoryQueryParams>,
    _path: MatchedPath,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Response, RpcError> {
    let project_id = query.project_id.clone();

    // TODO: Remove this once Dune Rootstock support is fixed
    // Return an empty history response for Rootstock until then
    // Cover Rootstock mainnet and testnet
    if query.chain_id.as_deref().is_some_and(|chain_id| {
        chain_id == ROOTSTOCK_MAINNET_CHAIN_ID || chain_id == ROOTSTOCK_TESTNET_CHAIN_ID
    }) {
        debug!("Temporary responding with an empty history response for Rootstock");
        return Ok(Json(HistoryResponseBody {
            data: vec![],
            next: None,
        })
        .into_response());
    }

    // If the chainId is not provided, then default to the Ethereum namespace
    let namespace = query
        .chain_id
        .as_ref()
        .map(|chain_id| {
            crypto::disassemble_caip2(chain_id)
                .map(|(namespace, _)| namespace)
                .unwrap_or(crypto::CaipNamespaces::Eip155)
        })
        .unwrap_or(crypto::CaipNamespaces::Eip155);

    if !crypto::is_address_valid(&address, &namespace) {
        return Err(RpcError::InvalidAddress);
    }

    let latency_tracker_start = std::time::SystemTime::now();
    let history_provider_kind: ProviderKind;
    let response: HistoryResponseBody = if let Some(onramp) = query.onramp.clone() {
        if onramp == "coinbase" && namespace == crypto::CaipNamespaces::Eip155 {
            // We don't want to validate the quota for the onramp
            state.validate_project_access(&project_id).await?;
            history_provider_kind = ProviderKind::Coinbase;
            state
                .providers
                .coinbase_pay_provider
                .get_transactions(
                    address.clone(),
                    query.clone().0,
                    &state.providers.token_metadata_cache,
                    state.metrics.clone(),
                )
                .await
                .tap_err(|e| {
                    error!("Failed to call coinbase transactions history with {e}");
                })?
        } else {
            return Err(RpcError::UnsupportedProvider(onramp));
        }
    } else {
        state.validate_project_access_and_quota(&project_id).await?;
        let provider = state
            .providers
            .history_providers
            .get(&namespace)
            .ok_or_else(|| RpcError::UnsupportedNamespace(namespace))?;
        history_provider_kind = provider.provider_kind();
        provider
            .get_transactions(
                address.clone(),
                query.0.clone(),
                &state.providers.token_metadata_cache,
                state.metrics.clone(),
            )
            .await
            .tap_err(|e| {
                error!("Failed to call transactions history with {e}");
            })?
    };

    let latency_tracker = latency_tracker_start
        .elapsed()
        .unwrap_or(std::time::Duration::from_secs(0));
    state.metrics.add_history_lookup(&history_provider_kind);

    let origin = headers
        .get("origin")
        .map(|v| v.to_str().unwrap_or("invalid_header").to_string());

    let (country, continent, region) = state
        .analytics
        .lookup_geo_data(network::get_forwarded_ip(&headers).unwrap_or_else(|| connect_info.0.ip()))
        .map(|geo| (geo.country, geo.continent, geo.region))
        .unwrap_or((None, None, None));

    // Filling the request_id from the `propagate_x_request_id` middleware
    let request_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown");

    // Analytics schema exception for Coinbase Onramp
    match history_provider_kind {
        ProviderKind::Coinbase => {
            for transaction in response.clone().data {
                state
                    .analytics
                    .onramp_history_lookup(OnrampHistoryLookupInfo::new(
                        transaction.id,
                        latency_tracker,
                        address.clone(),
                        project_id.clone(),
                        origin.clone(),
                        region.clone(),
                        country.clone(),
                        continent.clone(),
                        transaction.metadata.status,
                        transaction
                            .transfers
                            .as_ref()
                            .map(|v| {
                                v.first()
                                    .and_then(|item| {
                                        item.fungible_info.as_ref().map(|info| info.name.clone())
                                    })
                                    .unwrap_or_default()
                            })
                            .unwrap_or(None)
                            .unwrap_or_default(),
                        transaction.metadata.chain.clone().unwrap_or_default(),
                        transaction
                            .transfers
                            .as_ref()
                            .map(|v| v[0].quantity.numeric.clone())
                            .unwrap_or_default(),
                        query.sdk_info.sv.clone(),
                        query.sdk_info.st.clone(),
                        request_id.to_string(),
                    ));
            }
        }
        _ => {
            state.analytics.history_lookup(HistoryLookupInfo::new(
                address,
                project_id,
                response.data.len(),
                latency_tracker,
                response
                    .data
                    .iter()
                    .map(|transaction| transaction.transfers.as_ref().map(|v| v.len()).unwrap_or(0))
                    .sum(),
                response
                    .data
                    .iter()
                    .map(|transaction| {
                        transaction
                            .transfers
                            .as_ref()
                            .map(|v| {
                                v.iter()
                                    .filter(|transfer| transfer.fungible_info.is_some())
                                    .count()
                            })
                            .unwrap_or(0)
                    })
                    .sum(),
                response
                    .data
                    .iter()
                    .map(|transaction| {
                        transaction
                            .transfers
                            .as_ref()
                            .map(|v| {
                                v.iter()
                                    .filter(|transfer| transfer.nft_info.is_some())
                                    .count()
                            })
                            .unwrap_or(0)
                    })
                    .sum(),
                &history_provider_kind,
                origin,
                region,
                country,
                continent,
                query.sdk_info.sv.clone(),
                query.sdk_info.st.clone(),
                request_id.to_string(),
            ));
        }
    }

    let latency_tracker = latency_tracker_start
        .elapsed()
        .unwrap_or(std::time::Duration::from_secs(0));
    state
        .metrics
        .add_history_lookup_success(&history_provider_kind);
    state
        .metrics
        .add_history_lookup_latency(&history_provider_kind, latency_tracker);

    Ok(Json(response).into_response())
}
