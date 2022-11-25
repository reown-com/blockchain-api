use crate::handlers::RpcQueryParams;
use crate::json_rpc::JsonRpcRequest;
use parquet_derive::ParquetRecordWriter;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    timestamp: String,

    project_id: Arc<str>,
    chain_id: Arc<str>,
    method: Arc<str>,

    sender: Option<Arc<str>>,

    country: Option<Arc<str>>,
    continent: Option<Arc<str>>,
}

impl MessageInfo {
    pub fn new(
        query_params: &RpcQueryParams,
        request: &JsonRpcRequest,
        sender: Option<SocketAddr>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
    ) -> Self {
        Self {
            timestamp: super::create_timestamp(),

            project_id: Arc::from(query_params.project_id.to_owned()),
            chain_id: Arc::from(query_params.chain_id.to_lowercase()),
            method: request.method.clone(),

            sender: sender.map(|s| Arc::from(s.to_string())),

            country,
            continent,
        }
    }
}
