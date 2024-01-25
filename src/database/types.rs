use {
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize},
    sqlx::{FromRow, Type},
    std::collections::HashMap,
};

/// Currently supported blockchain namespaces
#[derive(Type, Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[sqlx(type_name = "namespaces", rename_all = "lowercase")]
pub enum SupportedNamespaces {
    /// Ethereum
    Eip155,
}

impl SupportedNamespaces {
    // Convert a SLIP-44 coin type to the SupportedNamespaces enum
    pub fn from_slip44(coin_type: u32) -> Option<SupportedNamespaces> {
        match coin_type {
            60 => Some(SupportedNamespaces::Eip155),
            _ => None,
        }
    }

    // Convert from the enum to the SLIP-44 coin type
    pub fn to_slip44(&self) -> u32 {
        match self {
            SupportedNamespaces::Eip155 => 60,
        }
    }
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Represents the ENSIP-11 compatible addresses map
pub type ENSIP11AddressesMap = HashMap<u32, Address>;

/// Represents the ENS name record
#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct NameAndAddresses {
    pub name: String,
    pub registered_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Postgres hstore data type, represented as key-value pairs for attributes
    pub attributes: Option<sqlx::types::Json<HashMap<String, String>>>,
    pub addresses: ENSIP11AddressesMap,
}
