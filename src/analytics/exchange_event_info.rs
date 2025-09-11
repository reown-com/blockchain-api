use parquet_derive::ParquetRecordWriter;
use serde::Serialize;
use strum_macros::Display;

#[derive(Debug, Clone, Copy, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ExchangeEventType {
    Started,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeEventInfo {
    pub timestamp: chrono::NaiveDateTime,
    pub event: String,

    pub id: String,
    pub exchange_id: String,
    pub project_id: String,

    pub asset: String,
    pub amount: f64,
    pub recipient: String,
    pub pay_url: String,

    pub tx_hash: Option<String>,
    pub failure_reason: Option<String>,
}

impl ExchangeEventInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event: ExchangeEventType,
        id: String,
        exchange_id: String,
        project_id: Option<String>,
        asset: Option<String>,
        amount: Option<f64>,
        recipient: Option<String>,
        pay_url: Option<String>,
        tx_hash: Option<String>,
        failure_reason: Option<String>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            event: event.to_string(),
            id,
            exchange_id,
            project_id: project_id.unwrap_or_default(),
            asset: asset.unwrap_or_default(),
            amount: amount.unwrap_or_default(),
            recipient: recipient.unwrap_or_default(),
            pay_url: pay_url.unwrap_or_default(),
            tx_hash,
            failure_reason,
        }
    }
}
