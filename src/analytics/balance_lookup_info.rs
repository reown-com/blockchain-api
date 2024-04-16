use {
    parquet_derive::ParquetRecordWriter,
    serde::Serialize,
    std::{sync::Arc, time::Duration},
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct BalanceLookupInfo {
    pub timestamp: chrono::NaiveDateTime,
    pub latency_secs: f64,

    pub symbol: String,
    pub quantity: String,

    pub address: String,
    pub project_id: String,

    pub origin: Option<String>,
    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,
}

impl BalanceLookupInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        latency: Duration,
        symbol: String,
        quantity: String,
        address: String,
        project_id: String,
        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            latency_secs: latency.as_secs_f64(),
            symbol,
            quantity,
            address,
            project_id,
            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,
        }
    }
}
