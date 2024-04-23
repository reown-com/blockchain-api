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
    pub implementation_chain_id: String,
    pub quantity: String,
    pub value: f64,
    pub price: f64,
    pub currency: String,

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
        implementation_chain_id: String,
        quantity: String,
        value: f64,
        price: f64,
        currency: String,
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
            implementation_chain_id,
            quantity,
            value,
            price,
            currency,
            address,
            project_id,
            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,
        }
    }
}
