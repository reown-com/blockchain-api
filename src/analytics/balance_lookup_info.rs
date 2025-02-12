use {
    crate::providers::ProviderKind, parquet_derive::ParquetRecordWriter, serde::Serialize,
    std::sync::Arc,
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct BalanceLookupInfo {
    pub timestamp: chrono::NaiveDateTime,

    pub symbol: String,
    pub implementation_chain_id: String,
    pub quantity: String,
    pub value: f64,
    pub price: f64,
    pub currency: String,

    pub address: String,
    pub project_id: String,

    pub provider: String,

    pub origin: Option<String>,
    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,

    // Sdk info
    pub sv: Option<String>,
    pub st: Option<String>,
}

impl BalanceLookupInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        implementation_chain_id: String,
        quantity: String,
        value: f64,
        price: f64,
        currency: String,
        address: String,
        project_id: String,
        provider: &ProviderKind,
        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
        sv: Option<String>,
        st: Option<String>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            symbol,
            implementation_chain_id,
            quantity,
            value,
            price,
            currency,
            address,
            project_id,
            provider: provider.to_string(),
            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,
            sv,
            st,
        }
    }
}
