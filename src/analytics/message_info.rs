use {
    crate::{handlers::RpcQueryParams, json_rpc::JsonRpcRequest, providers::ProviderKind},
    parquet_derive::ParquetRecordWriter,
    serde::Serialize,
    std::sync::Arc,
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    pub timestamp: chrono::NaiveDateTime,

    pub project_id: String,
    pub chain_id: String,
    pub method: Arc<str>,

    pub origin: Option<String>,
    pub provider: String,

    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,
}

impl MessageInfo {
    pub fn new(
        query_params: &RpcQueryParams,
        request: &JsonRpcRequest,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
        provider: &ProviderKind,
        origin: Option<String>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),

            project_id: query_params.project_id.to_owned(),
            chain_id: query_params.chain_id.to_lowercase(),
            method: request.method.clone(),

            origin,
            provider: provider.to_string(),

            region: region.map(|r| r.join(", ")),
            country,
            continent,
        }
    }
}
