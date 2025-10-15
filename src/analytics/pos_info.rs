use {
    crate::handlers::json_rpc::pos::TransactionStatus,
    parquet_derive::ParquetRecordWriter,
    serde::Serialize,
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct PosBuildTxInfo {
    pub timestamp: chrono::NaiveDateTime,

    pub project_id: String,

    pub asset: String,
    pub amount: String,
    pub recipient: String,
    pub sender: String,
    pub capabilities: Option<String>,

    pub transaction_id: String,
    pub tx_chain_id: String,
    pub tx_method: String,
    pub tx_params: String,
}

impl PosBuildTxInfo {
    pub fn new(input: PosBuildTxNew) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            project_id: input.project_id.to_string(),
            asset: input.request.asset.to_string(),
            amount: input.request.amount.to_string(),
            recipient: input.request.recipient.to_string(),
            sender: input.request.sender.to_string(),
            capabilities: input.request.capabilities.map(str::to_owned),
            transaction_id: input.response.transaction_id.to_string(),
            tx_chain_id: input.response.tx_chain_id.to_string(),
            tx_method: input.response.tx_method.to_string(),
            tx_params: input.response.tx_params.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PosBuildTxRequest<'a> {
    pub asset: &'a str,
    pub amount: &'a str,
    pub recipient: &'a str,
    pub sender: &'a str,
    pub capabilities: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct PosBuildTxResponse<'a> {
    pub transaction_id: &'a str,
    pub tx_chain_id: &'a str,
    pub tx_method: &'a str,
    pub tx_params: &'a str,
}

#[derive(Debug, Clone)]
pub struct PosBuildTxNew<'a> {
    pub project_id: &'a str,
    pub request: PosBuildTxRequest<'a>,
    pub response: PosBuildTxResponse<'a>,
}

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct PosCheckTxInfo {
    pub timestamp: chrono::NaiveDateTime,

    pub project_id: String,
    pub chain_id: String,
    pub transaction_id: String,
    pub send_result: String,

    pub status: String,
    pub check_in: Option<usize>,
    pub tx_hash: Option<String>,
}

impl PosCheckTxInfo {
    pub fn new(
        project_id: String,
        chain_id: String,
        transaction_id: String,
        send_result: String,
        status: &TransactionStatus,
        check_in: Option<usize>,
        tx_hash: Option<String>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            project_id,
            chain_id,
            transaction_id,
            send_result,
            status: status.to_string(),
            check_in,
            tx_hash,
        }
    }
}
