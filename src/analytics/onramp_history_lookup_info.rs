use {
    parquet_derive::ParquetRecordWriter,
    serde::Serialize,
    std::{sync::Arc, time::Duration},
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
pub struct OnrampHistoryLookupInfo {
    pub timestamp: chrono::NaiveDateTime,
    pub transaction_id: String,
    pub latency_secs: f64,

    pub lookup_address: String,
    pub project_id: String,

    pub origin: Option<String>,
    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,

    pub transaction_status: String,
    pub purchase_currency: String,
    pub purchase_network: String,
    pub purchase_amount: String,
}

impl OnrampHistoryLookupInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transaction_id: String,
        latency: Duration,
        lookup_address: String,
        project_id: String,
        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,

        transaction_status: String,
        purchase_currency: String,
        purchase_network: String,
        purchase_amount: String,
    ) -> Self {
        OnrampHistoryLookupInfo {
            transaction_id,
            timestamp: wc::analytics::time::now(),
            latency_secs: latency.as_secs_f64(),
            lookup_address,
            project_id,
            origin,
            country,
            region: region.map(|r| r.join(", ")),
            continent,

            transaction_status,
            purchase_currency,
            purchase_network,
            purchase_amount,
        }
    }
}
