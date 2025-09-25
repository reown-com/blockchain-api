use {
    crate::handlers::json_rpc::pos::TransactionStatus, parquet_derive::ParquetRecordWriter,
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

    pub tx_id: String,
    pub tx_chain_id: String,
    pub tx_method: String,
    pub tx_params: String,
}

impl PosBuildTxInfo {
    pub fn new(input: PosBuildTxNew) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            project_id: input.project_id.to_owned(),
            asset: input.request.asset.to_owned(),
            amount: input.request.amount.to_owned(),
            recipient: input.request.recipient.to_owned(),
            sender: input.request.sender.to_owned(),
            capabilities: input.request.capabilities.map(str::to_owned),
            tx_id: input.response.tx_id.to_owned(),
            tx_chain_id: input.response.tx_chain_id.to_owned(),
            tx_method: input.response.tx_method.to_owned(),
            tx_params: input.response.tx_params.to_owned(),
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
    pub tx_id: &'a str,
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
    pub txid: Option<String>,
}

impl PosCheckTxInfo {
    pub fn new(
        project_id: String,
        chain_id: String,
        transaction_id: String,
        send_result: String,
        status: &TransactionStatus,
        check_in: Option<usize>,
        txid: Option<String>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            project_id,
            chain_id,
            transaction_id,
            send_result,
            status: match status {
                TransactionStatus::Pending => "PENDING".to_string(),
                TransactionStatus::Confirmed => "CONFIRMED".to_string(),
                TransactionStatus::Failed => "FAILED".to_string(),
            },
            check_in,
            txid,
        }
    }
}
