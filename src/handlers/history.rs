use {
    super::{HistoryResponseBody, HANDLER_TASK_METRICS},
    crate::{
        analytics::HistoryLookupInfo,
        error::RpcError,
        handlers::HistoryQueryParams,
        state::AppState,
        utils::network,
    },
    axum::{
        body::Bytes,
        extract::{ConnectInfo, MatchedPath, Path, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    ethers::types::H160,
    hyper::HeaderMap,
    std::{net::SocketAddr, str::FromStr, sync::Arc},
    tap::TapFallible,
    tracing::log::error,
    wc::future::FutureExt,
};

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    query: Query<HistoryQueryParams>,
    path: MatchedPath,
    headers: HeaderMap,
    address: Path<String>,
    body: Bytes,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, query, path, headers, address, body)
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
    body: Bytes,
) -> Result<Response, RpcError> {
    let project_id = query.project_id.clone();
    let address_hash = H160::from_str(&address).map_err(|_| RpcError::IdentityInvalidAddress)?;

    state.validate_project_access(&project_id).await?;
    let latency_tracker_start = std::time::SystemTime::now();
    let mut response = state
        .providers
        .history_provider
        .get_transactions(address_hash, body.clone(), query.0.clone())
        .await
        .tap_err(|e| {
            error!("Failed to call transactions history with {}", e);
        })?;

    if let Some(_onramp) = query.onramp.clone() {
        let coinbase_transactions: HistoryResponseBody = state
            .providers
            .coinbase_pay_provider
            .get_transactions(address_hash, body, query.0)
            .await
            .tap_err(|e| {
                error!("Failed to call coinbase transactions history with {}", e);
            })
            .unwrap_or(HistoryResponseBody {
                data: Vec::default(),
                next: None,
            });

        // move this to the beginning of the transactions
        response.data.extend(coinbase_transactions.data);

        // now order all of this by `mined_at`
        response.data.sort_by(|a, b| {
            a.metadata
                .mined_at
                .partial_cmp(&b.metadata.mined_at)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let latency_tracker = latency_tracker_start
        .elapsed()
        .unwrap_or(std::time::Duration::from_secs(0));
    state.metrics.add_history_lookup();

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

        state.analytics.history_lookup(HistoryLookupInfo::new(
            address_hash.to_string(),
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

    let latency_tracker = latency_tracker_start
        .elapsed()
        .unwrap_or(std::time::Duration::from_secs(0));
    state.metrics.add_history_lookup_success();
    state.metrics.add_history_lookup_latency(latency_tracker);

    Ok(Json(response).into_response())
}
