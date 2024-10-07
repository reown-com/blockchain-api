use {
    crate::{handlers::RpcQueryParams, json_rpc::JsonRpcRequest, providers::ProviderKind},
    parquet_derive::ParquetRecordWriter,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    strum::{Display, EnumString},
};

#[derive(Debug, Clone, Serialize, ParquetRecordWriter)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    pub timestamp: chrono::NaiveDateTime,

    pub project_id: String,
    pub chain_id: String,
    pub method: Arc<str>,
    pub source: String,

    pub origin: Option<String>,
    pub provider: String,

    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,
}

impl MessageInfo {
    pub fn new(
        query_params: &RpcQueryParams,
        request: &JsonRpcRequest,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
        provider: &ProviderKind,
        origin: Option<String>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),

            project_id: query_params.project_id.to_owned(),
            chain_id: query_params.chain_id.to_lowercase(),
            method: request.method.clone(),
            source: query_params
                .source
                .as_ref()
                .unwrap_or(&MessageSource::Rpc)
                .to_string(),

            origin,
            provider: provider.to_string(),

            region: region.map(|r| r.join(", ")),
            country,
            continent,
        }
    }
}

#[derive(Debug, Clone, EnumString, Display, Deserialize, PartialEq)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MessageSource {
    Rpc,
    Identity,
    Balance,
    ProfileAddressSigValidate,
    ProfileAttributesSigValidate,
    ProfileRegisterSigValidate,
    SessionCoSignSigValidate,
    WalletPrepareCalls,
    WalletSendPreparedCalls,
    WalletGetCallsStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_message_source() {
        let source = MessageSource::Rpc;
        assert_eq!(source.to_string(), "rpc");

        let source = MessageSource::Identity;
        assert_eq!(source.to_string(), "identity");

        let source = MessageSource::Balance;
        assert_eq!(source.to_string(), "balance");

        let source = MessageSource::ProfileAddressSigValidate;
        assert_eq!(source.to_string(), "profile_address_sig_validate");

        let source = MessageSource::ProfileAttributesSigValidate;
        assert_eq!(source.to_string(), "profile_attributes_sig_validate");

        let source = MessageSource::ProfileRegisterSigValidate;
        assert_eq!(source.to_string(), "profile_register_sig_validate");

        let source = MessageSource::SessionCoSignSigValidate;
        assert_eq!(source.to_string(), "session_co_sign_sig_validate");
    }

    #[test]
    fn deserialize_message_source() {
        let source = serde_json::json!("rpc");
        assert_eq!(
            serde_json::from_value::<MessageSource>(source).unwrap(),
            MessageSource::Rpc
        );
    }
}
