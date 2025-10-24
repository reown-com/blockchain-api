use {
    crate::providers::ProviderKind,
    parquet_derive::ParquetRecordWriter,
    serde::Serialize,
    std::{sync::Arc, time::Duration},
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct HistoryLookupInfo {
    pub timestamp: chrono::NaiveDateTime,

    pub lookup_address: String,
    pub project_id: String,

    pub transactions_count: usize,
    pub latency_secs: f64,

    pub transfers_count: usize,
    pub fungibles_count: usize,
    pub nft_count: usize,

    pub provider: String,

    pub origin: Option<String>,
    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,

    // Sdk info
    pub sv: Option<String>,
    pub st: Option<String>,

    pub request_id: String,
}

impl HistoryLookupInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lookup_address: String,
        project_id: String,
        transactions_count: usize,
        latency: Duration,
        transfers_count: usize,
        fungibles_count: usize,
        nft_count: usize,
        provider: &ProviderKind,
        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
        sv: Option<String>,
        st: Option<String>,
        request_id: String,
    ) -> Self {
        HistoryLookupInfo {
            timestamp: wc::analytics::time::now(),
            lookup_address,
            project_id,
            transactions_count,
            latency_secs: latency.as_secs_f64(),
            transfers_count,
            fungibles_count,
            nft_count,
            provider: provider.to_string(),
            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,
            sv,
            st,
            request_id,
        }
    }
}
