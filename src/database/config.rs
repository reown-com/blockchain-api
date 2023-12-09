use serde::Deserialize;

const DEFAULT_MAX_CONNECTIONS: u16 = 10;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PostgresConfig {
    /// The database connection uri.
    /// postgres://postgres@localhost:5432/postgres
    pub uri: String,
    /// Maximum connections for the sqlx pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u16,
}

fn default_max_connections() -> u16 {
    DEFAULT_MAX_CONNECTIONS
}
