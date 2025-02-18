use crate::{
    error::RpcError,
    handlers::{
        self,
        balance::{BalanceQueryParams, BalanceResponseBody},
        SdkInfoParams, SupportedCurrencies, HANDLER_TASK_METRICS,
    },
    state::AppState,
};
use alloy::primitives::{Address, BlockHash, Bytes, TxHash, B256, U64, U8};
use axum::{
    extract::{ConnectInfo, Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use thiserror::Error;
use tracing::error;
use wc::future::FutureExt;

// https://github.com/ethereum/ERCs/pull/709/files#diff-be675f3ce6b6aa5616dd1bccf5e50f44ad65775afb967a47aaffb8f5eb51b849R35
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsParams {
    account: Address,
    // TODO asset_filter, asset_type_filter, chain_filter
}

pub type Eip155ChainId = U64;
pub type GetAssetsResult = HashMap<Eip155ChainId, Asset>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum Asset {
    Native {
        #[serde(flatten)]
        data: AssetData<NativeMetadata>,
    },
    Erc20 {
        #[serde(flatten)]
        data: AssetData<Erc20Metadata>,
    },
    Erc721 {
        #[serde(flatten)]
        data: AssetData<Erc721Metadata>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetData<M> {
    address: String,
    balance: U64,
    metadata: M,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeMetadata {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc20Metadata {
    name: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc721Metadata {
    name: String,
    symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CallStatus {
    Pending,
    Confirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallReceipt {
    logs: Vec<CallReceiptLog>,
    status: U8,
    chain_id: U64,
    block_hash: BlockHash,
    block_number: U64,
    gas_used: U64,
    transaction_hash: TxHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallReceiptLog {
    address: Address,
    data: Bytes,
    topics: Vec<B256>,
}

#[derive(Error, Debug)]
pub enum GetAssetsError {
    #[error("Internal error")]
    InternalError(GetAssetsErrorInternalError),
}

#[derive(Error, Debug)]
pub enum GetAssetsErrorInternalError {
    #[error("GetBalance call failed: {0}")]
    GetBalance(RpcError),
}

impl IntoResponse for GetAssetsError {
    fn into_response(self) -> Response {
        #[allow(unreachable_patterns)] // TODO remove
        match self {
            Self::InternalError(e) => {
                error!("HTTP server error: (get_assets) {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            e => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string(),
                })),
            )
                .into_response(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    request: GetAssetsParams,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
) -> Result<GetAssetsResult, GetAssetsError> {
    handler_internal(state, project_id, request, connect_info, headers, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("wallet_get_assets"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    project_id: String,
    request: GetAssetsParams,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
) -> Result<GetAssetsResult, GetAssetsError> {
    let balance = handlers::balance::handler(
        state,
        Query(BalanceQueryParams {
            project_id,
            currency: SupportedCurrencies::USD,
            chain_id: None,
            force_update: None,
            sdk_info: query.sdk_info.clone(),
        }),
        ConnectInfo(connect_info),
        headers,
        Path(request.account.to_string()),
    )
    .await
    .map_err(|e| GetAssetsError::InternalError(GetAssetsErrorInternalError::GetBalance(e)))?;

    get_assets(balance.0)
}

fn get_assets(_balance: BalanceResponseBody) -> Result<GetAssetsResult, GetAssetsError> {
    Ok(GetAssetsResult::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_assets() {
        let balance = BalanceResponseBody { balances: vec![] };
        let assets = get_assets(balance).unwrap();
        assert!(assets.is_empty());
    }
}
