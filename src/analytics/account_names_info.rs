use {parquet_derive::ParquetRecordWriter, serde::Serialize, std::sync::Arc};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct AccountNameRegistration {
    pub timestamp: chrono::NaiveDateTime,

    pub name: String,
    pub owner_address: String,
    pub chain_id: String,

    pub origin: Option<String>,
    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,
}

impl AccountNameRegistration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        owner_address: String,
        chain_id: String,
        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            name,
            owner_address,
            chain_id,
            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,
        }
    }
}
