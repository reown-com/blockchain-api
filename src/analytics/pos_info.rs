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
    pub fn new(
        project_id: String,
        asset: String,
        amount: String,
        recipient: String,
        sender: String,
        capabilities: Option<String>,
        tx_id: String,
        tx_chain_id: String,
        tx_method: String,
        tx_params: String,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),
            project_id,
            asset,
            amount,
            recipient,
            sender,
            capabilities,
            tx_id,
            tx_chain_id,
            tx_method,
            tx_params,
        }
    }
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
