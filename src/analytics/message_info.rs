use {
    crate::{handlers::RpcQueryParams, providers::ProviderKind},
    hyper::HeaderMap,
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
    pub method: String,
    pub source: String,

    pub request_id: Option<String>,
    pub rpc_id: String,
    pub session_id: Option<String>,

    pub origin: Option<Arc<str>>,
    pub provider: String,

    pub region: Option<String>,
    pub country: Option<Arc<str>>,
    pub continent: Option<Arc<str>>,

    // Sdk info
    pub sv: Option<String>,
    pub st: Option<String>,
}

impl MessageInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        query_params: &RpcQueryParams,
        headers: &HeaderMap,
        session_id: Option<String>,
        rpc_id: String,
        rpc_method: String,
        region: Option<Vec<String>>,
        country: Option<Arc<str>>,
        continent: Option<Arc<str>>,
        provider: &ProviderKind,
        origin: Option<Arc<str>>,
        sv: Option<String>,
        st: Option<String>,
    ) -> Self {
        Self {
            timestamp: wc::analytics::time::now(),

            project_id: query_params.project_id.to_owned(),
            chain_id: query_params.chain_id.clone(),
            method: rpc_method,
            source: query_params
                .source
                .as_ref()
                .unwrap_or(&MessageSource::Rpc)
                .to_string(),

            request_id: headers
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string()),
            rpc_id,
            session_id,

            origin,
            provider: provider.to_string(),

            region: region.map(|r| r.join(", ")),
            country,
            continent,
            sv,
            st,
        }
    }
}

// Note: these are all INTERNAL sources (except Rpc). While technically the user
// can override this via query param currently, this is just a technical
// limitation of the implementation here.
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
    WalletGetAssets,
    ChainAgnosticCheck,
    WalletBuildPosTx,
    WalletSendPosTx,
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

        let source = MessageSource::ChainAgnosticCheck;
        assert_eq!(source.to_string(), "chain_agnostic_check");

        let source = MessageSource::WalletBuildPosTx;
        assert_eq!(source.to_string(), "wallet_build_pos_tx");

        let source = MessageSource::WalletSendPosTx;
        assert_eq!(source.to_string(), "wallet_send_pos_tx");
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
