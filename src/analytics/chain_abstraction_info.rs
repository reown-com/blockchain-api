use {parquet_derive::ParquetRecordWriter, serde::Serialize, std::sync::Arc};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
pub struct ChainAbstractionFundingInfo {
    pub timestamp: chrono::NaiveDateTime,
    pub project_id: String,

    pub origin: Option<String>,
    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,

    pub chain_id: String,
    pub token_contract: String,
    pub token_symbol: String,
    pub amount: String,
}

impl ChainAbstractionFundingInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: String,

        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,

        chain_id: String,
        token_contract: String,
        token_symbol: String,
        amount: String,
    ) -> Self {
        ChainAbstractionFundingInfo {
            timestamp: wc::analytics::time::now(),
            project_id,

            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,

            chain_id,
            token_contract,
            token_symbol,
            amount,
        }
    }
}

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
pub struct ChainAbstractionBridgingInfo {
    pub timestamp: chrono::NaiveDateTime,
    pub project_id: String,

    pub origin: Option<String>,
    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,

    pub src_chain_id: String,
    pub src_token_contract: String,
    pub src_token_symbol: String,

    pub dst_chain_id: String,
    pub dst_token_contract: String,
    pub dst_token_symbol: String,

    pub amount: String,
    pub bridging_fee: String,
}

impl ChainAbstractionBridgingInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: String,

        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,

        src_chain_id: String,
        src_token_contract: String,
        src_token_symbol: String,

        dst_chain_id: String,
        dst_token_contract: String,
        dst_token_symbol: String,

        amount: String,
        bridging_fee: String,
    ) -> Self {
        ChainAbstractionBridgingInfo {
            timestamp: wc::analytics::time::now(),
            project_id,

            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,

            src_chain_id,
            src_token_contract,
            src_token_symbol,

            dst_chain_id,
            dst_token_contract,
            dst_token_symbol,

            amount,
            bridging_fee,
        }
    }
}

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
pub struct ChainAbstractionInitialTxInfo {
    pub timestamp: chrono::NaiveDateTime,
    pub project_id: String,

    pub origin: Option<String>,
    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,

    pub transfer_from: String,
    pub transfer_to: String,
    pub amount: String,
    pub chain_id: String,
    pub token_contract: String,
    pub token_symbol: String,
}

impl ChainAbstractionInitialTxInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: String,

        origin: Option<String>,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,

        transfer_from: String,
        transfer_to: String,
        amount: String,
        chain_id: String,
        token_contract: String,
        token_symbol: String,
    ) -> Self {
        ChainAbstractionInitialTxInfo {
            timestamp: wc::analytics::time::now(),
            project_id,

            origin,
            region: region.map(|r| r.join(", ")),
            country,
            continent,

            transfer_from,
            transfer_to,
            amount,
            chain_id,
            token_contract,
            token_symbol,
        }
    }
}
