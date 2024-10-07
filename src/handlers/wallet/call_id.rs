use alloy::primitives::{Bytes, B256, U64};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallId(pub CallIdInner);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallIdInner {
    pub chain_id: U64,
    pub user_op_hash: B256,
}

impl Serialize for CallId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Bytes::from(serde_json::to_vec(&self.0).map_err(serde::ser::Error::custom)?)
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CallId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = Bytes::deserialize(deserializer)?;
        let inner = serde_json::from_slice(&bytes).map_err(serde::de::Error::custom)?;
        Ok(Self(inner))
    }
}
