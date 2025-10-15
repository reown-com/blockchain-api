use {
    alloy::primitives::U64,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureRequestType {
    #[serde(rename = "user-operation-v07")]
    UserOpV7,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedCalls {
    pub r#type: SignatureRequestType,
    pub data: yttrium::user_operation::UserOperationV07,
    pub chain_id: U64,
}
