use {
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize, Serializer},
    sqlx::{FromRow, Type},
    std::collections::HashMap,
};

/// Currently supported blockchain namespaces
#[derive(Type, Serialize, Deserialize, Debug, Clone)]
#[sqlx(type_name = "namespaces", rename_all = "lowercase")]
pub enum SupportedNamespaces {
    /// Ethereum
    Eip155,
}

/// Represents the ENS name record
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
    pub registered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Postgres hstore data type, represented as key-value pairs for attributes
    pub attributes: Option<sqlx::types::Json<HashMap<String, String>>>,
}

/// Represents the ENS address record
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Address {
    pub namespace: SupportedNamespaces,
    #[serde(serialize_with = "serialize_chain_id")]
    pub chain_id: Option<String>,
    pub address: String,
    pub created_at: Option<DateTime<Utc>>,
}

// Custom serialization function for chain_id to make it None if it's an empty
// string
fn serialize_chain_id<S>(chain_id: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match chain_id {
        Some(id) if id.is_empty() => serializer.serialize_none(),
        _ => serializer.serialize_some(chain_id),
    }
}

/// Represents the ENS name record
#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct NameAndAddresses {
    pub name: String,
    pub registered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Postgres hstore data type, represented as key-value pairs for attributes
    pub attributes: Option<sqlx::types::Json<HashMap<String, String>>>,
    pub addresses: Vec<Address>,
}
