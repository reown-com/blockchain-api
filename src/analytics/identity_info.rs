use {
    crate::{handlers::RpcQueryParams, json_rpc::JsonRpcRequest, providers::ProviderKind},
    parquet_derive::ParquetRecordWriter,
    serde::Serialize,
    std::sync::Arc,
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct IdentityInfo {
    pub timestamp: chrono::NaiveDateTime,

    pub project_id: String,
    pub chain_id: String,
    pub account: String,

    pub origin: Option<String>,
    pub provider: Option<String>,

    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,
}

impl IdentityInfo {
    pub fn new(
        query_params: &RpcQueryParams,
        request: &JsonRpcRequest,
        account: Arc<str>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
        provider: &Option<ProviderKind>,
        origin: Option<String>,
    ) -> Self {
        Self {
            timestamp: gorgon::time::now(),

            project_id: query_params.project_id.to_owned(),
            chain_id: query_params.chain_id.to_lowercase(),
            account: account.to_string(),

            origin,
            provider: provider.to_string(),

            region: region.map(|r| r.join(", ")),
            country,
            continent,
        }
    }
}
