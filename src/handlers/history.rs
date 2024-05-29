use {
    super::HANDLER_TASK_METRICS,
    crate::{
        analytics::{HistoryLookupInfo, OnrampHistoryLookupInfo},
        error::RpcError,
        providers::ProviderKind,
        state::AppState,
        utils::network,
    },
    axum::{
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::types::H160,
    hyper::HeaderMap,
    serde::{Deserialize, Serialize},
    std::{net::SocketAddr, str::FromStr, sync::Arc},
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQueryParams {
    pub currency: Option<String>,
    pub project_id: String,
    pub cursor: Option<String>,
    pub onramp: Option<String>,
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
        .with_metrics(HANDLER_TASK_METRICS.with_name("transactions"))
        .await
}

#[tracing::instrument(skip_all)]
async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<HistoryQueryParams>,
    _path: MatchedPath,
    headers: HeaderMap,
    Path(address): Path<String>,
) -> Result<Response, RpcError> {
    let project_id = query.project_id.clone();

    // Checking for the H160 address correctness
    H160::from_str(&address).map_err(|_| RpcError::InvalidAddress)?;

    let latency_tracker_start = std::time::SystemTime::now();
    let history_provider: ProviderKind;
    let response: HistoryResponseBody = if let Some(onramp) = query.onramp.clone() {
        if onramp == "coinbase" {
            // We don't want to validate the quota for the onramp
            state.validate_project_access(&project_id).await?;
            history_provider = ProviderKind::Coinbase;
            state
                .providers
                .coinbase_pay_provider
                .get_transactions(address.clone(), query.clone().0, state.http_client.clone())
                .await
                .tap_err(|e| {
                    error!("Failed to call coinbase transactions history with {}", e);
                })?
        } else {
            return Err(RpcError::UnsupportedProvider(onramp));
        }
    } else {
        state.validate_project_access_and_quota(&project_id).await?;
        history_provider = ProviderKind::Zerion;
        state
            .providers
            .history_provider
            .get_transactions(address.clone(), query.0.clone(), state.http_client.clone())
            .await
            .tap_err(|e| {
                error!("Failed to call transactions history with {}", e);
            })?
    };

    let latency_tracker = latency_tracker_start
        .elapsed()
        .unwrap_or(std::time::Duration::from_secs(0));
    state.metrics.add_history_lookup(&history_provider);

    let origin = headers
        .get("origin")
        .map(|v| v.to_str().unwrap_or("invalid_header").to_string());

    let (country, continent, region) = state
        .analytics
        .lookup_geo_data(network::get_forwarded_ip(headers).unwrap_or_else(|| connect_info.0.ip()))
        .map(|geo| (geo.country, geo.continent, geo.region))
        .unwrap_or((None, None, None));

    // Different analytics for different history providers
    match history_provider {
        ProviderKind::Zerion => {
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
                origin,
                region,
                country,
                continent,
            ));
        }
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
                    ));
            }
        }
        _ => {}
    }

    let latency_tracker = latency_tracker_start
        .elapsed()
        .unwrap_or(std::time::Duration::from_secs(0));
    state.metrics.add_history_lookup_success(&history_provider);
    state
        .metrics
        .add_history_lookup_latency(&history_provider, latency_tracker);

    Ok(Json(response).into_response())
}
