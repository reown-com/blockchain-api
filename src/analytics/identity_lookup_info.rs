use {
    crate::handlers::identity::{IdentityLookupSource, IdentityQueryParams, ETHEREUM_MAINNET},
    ethers::types::H160,
    parquet_derive::ParquetRecordWriter,
    serde::Serialize,
    std::{sync::Arc, time::Duration},
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct IdentityLookupInfo {
    pub timestamp: chrono::NaiveDateTime,

    pub address_hash: String,
    pub name_present: bool,
    pub avatar_present: bool,
    pub source: String,
    pub latency_secs: f64,

    pub project_id: String,
    pub chain_id: String,

    pub origin: Option<String>,

    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,
}

impl IdentityLookupInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        query_params: &IdentityQueryParams,
        address: H160,
        name_present: bool,
        avatar_present: bool,
        source: IdentityLookupSource,
        latency: Duration,
        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),

            address_hash: sha256::digest(address.as_ref()),
            name_present,
            avatar_present,
            source: source.as_str().to_string(),
            latency_secs: latency.as_secs_f64(),

            project_id: query_params.project_id.to_owned(),
            chain_id: ETHEREUM_MAINNET.to_owned(),

            origin,

            region: region.map(|r| r.join(", ")),
            country,
            continent,
        }
    }
}
