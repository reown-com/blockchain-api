use {
    crate::{
        project::ProjectDataError,
        storage::error::StorageError,
        utils::crypto::CryptoUitlsError,
    },
    axum::{response::IntoResponse, Json},
    cerberus::registry::RegistryError,
    hyper::StatusCode,
    tracing::{debug, log::error},
};

pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error(transparent)]
    EnvyError(#[from] envy::Error),

    #[error(transparent)]
    CryptoUitlsError(#[from] CryptoUitlsError),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Project data error: {0}")]
    ProjectDataError(#[from] ProjectDataError),

    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),

    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),

    #[error("Chain not found despite previous validation")]
    ChainNotFound,

    #[error("Transport error: {0}")]
    TransportError(#[from] hyper::Error),

    #[error("Request::builder() failed: {0}")]
    RequestBuilderError(#[from] hyper::http::Error),

    #[error("Specified chain is not supported by any of the providers: {0}")]
    UnsupportedChain(String),

    #[error("Requested chain provider is temporarily unavailable: {0}")]
    ChainTemporarilyUnavailable(String),

    #[error("Specified provider is not supported: {0}")]
    UnsupportedProvider(String),

    #[error("Failed to reach the provider")]
    ProviderError,

    #[error("Failed to reach the transaction provider")]
    TransactionProviderError,

    #[error("Failed to reach the portfolio provider")]
    PortfolioProviderError,

    #[error("Failed to reach the balance provider")]
    BalanceProviderError,

    #[error("Failed to reach the fungible price provider")]
    FungiblePriceProviderError,

    #[error("Failed to parse balance provider url")]
    BalanceParseURLError,

    #[error("Failed to parse fungible price provider url")]
    FungiblePriceParseURLError,

    #[error("Failed to parse onramp provider url")]
    OnRampParseURLError,

    #[error("Failed to reach the onramp provider")]
    OnRampProviderError,

    #[error(transparent)]
    Cerberus(#[from] cerberus::project::AccessError),

    #[error("{0:?}")]
    Other(#[from] anyhow::Error),

    #[error("{0:?}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Invalid scheme used. Try http(s):// or ws(s)://")]
    InvalidScheme,

    #[error(transparent)]
    AxumTungstenite(#[from] axum_tungstenite::Error),

    #[error(transparent)]
    RateLimited(#[from] wc::rate_limit::RateLimitExceeded),

    #[error("Invalid address")]
    InvalidAddress,

    #[error("Failed to parse provider cursor")]
    HistoryParseCursorError,

    #[error("Identity lookup error: {0}")]
    IdentityLookup(String),

    #[error("Quota limit reached")]
    QuotaLimitReached,

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::error::Error),

    #[error("sqlx migration error: {0}")]
    SqlxMigrationError(#[from] sqlx::migrate::MigrateError),

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    // Conversion errors
    #[error("Failed to reach the conversion provider")]
    ConversionProviderError,

    #[error("Failed to parse conversion provider url")]
    ConversionParseURLError,

    #[error("Invalid conversion parameter: {0}")]
    ConversionInvalidParameter(String),

    // Profile names errors
    #[error("Name is already registered: {0}")]
    NameAlreadyRegistered(String),

    #[error("Name is not registered: {0}")]
    NameNotRegistered(String),

    #[error("Name is not found: {0}")]
    NameNotFound(String),

    #[error("No name is found for address")]
    NameByAddressNotFound,

    #[error("Invalid name format: {0}")]
    InvalidNameFormat(String),

    #[error("Invalid name length: {0}")]
    InvalidNameLength(String),

    #[error("Name is not in the allowed zones: {0}")]
    InvalidNameZone(String),

    #[error("Unsupported coin type: {0}")]
    UnsupportedCoinType(u32),

    #[error("Unsupported name attribute")]
    UnsupportedNameAttribute,

    #[error("Signature UNIXTIME timestamp is too old: {0}")]
    ExpiredTimestamp(u64),

    #[error("Error during the signature validation: {0}")]
    SignatureValidationError(String),

    #[error("Name owner validation error")]
    NameOwnerValidationError,

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("Weighted providers index error: {0}")]
    WeightedProvidersIndex(String),
}

impl IntoResponse for RpcError {
    fn into_response(self) -> axum::response::Response {
        let response =  match &self {
            Self::AxumTungstenite(err) => (StatusCode::GONE, err.to_string()).into_response(),
            Self::UnsupportedChain(chain_id) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "chainId".to_string(),
                    format!("We don't support the chainId you provided: {chain_id}. See the list of supported chains here: https://docs.walletconnect.com/cloud/blockchain-api#supported-chains"),
                )),
            )
                .into_response(),
            Self::CryptoUitlsError(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "".to_string(),
                    format!("Crypto utils invalid argument: {}", e),
                )),
            )
                .into_response(),
            Self::ChainTemporarilyUnavailable(chain_id) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "chainId".to_string(),
                    format!("Requested {chain_id} chain provider is temporarily unavailable"),
                )),
            )
                .into_response(),
            Self::UnsupportedProvider(provider) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "provider".to_string(),
                    format!("Provider {provider} is not supported"),
                )),
            )
                .into_response(),
            Self::ProviderError => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "unreachable".to_string(),
                    "We failed to reach the provider for your request".to_string(),
                )),
            )
                .into_response(),
            Self::InvalidScheme => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "scheme".to_string(),
                    "Invalid scheme used. Try http(s):// or ws(s)://".to_string(),
                )),
            )
                .into_response(),
            Self::RegistryError(_) | Self::Cerberus(_) | Self::ProjectDataError(_) => (
                StatusCode::UNAUTHORIZED,
                Json(new_error_response(
                    "authentication".to_string(),
                    "We failed to authenticate your request".to_string(),
                )),
            )
                .into_response(),
            Self::TransportError(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "transport".to_string(),
                    "We failed to reach the provider for your request".to_string(),
                )),
            )
                .into_response(),
            Self::InvalidAddress => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "address".to_string(),
                    "The address provided is invalid".to_string(),
                )),
            )
                .into_response(),
            Self::QuotaLimitReached => (
                StatusCode::TOO_MANY_REQUESTS,
                Json(new_error_response(
                    "address".to_string(),
                    "Project's quota limit reached".to_string(),
                )),
            )
                .into_response(),
            Self::InvalidParameter(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "".to_string(),
                    format!("Invalid parameter: {}", e),
                )),
            )
                .into_response(),

            // Profile names errors
            Self::InvalidNameFormat(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Invalid name format: {}", e),
                )),
            )
                .into_response(),
            Self::InvalidNameLength(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Invalid name length: {}", e),
                )),
            )
                .into_response(),
            Self::InvalidNameZone(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Name is not in the allowed zones: {}", e),
                )),
            )
                .into_response(),
            Self::ConversionInvalidParameter(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(new_error_response(
                        "".to_string(),
                        format!("Conversion parameter error: {}", e),
                    )),
                )
                    .into_response(),
            Self::UnsupportedCoinType(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "coin_type".to_string(),
                    format!("Unsupported coin type: {}", e),
                )),
            )
                .into_response(),
            Self::UnsupportedNameAttribute => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "attributes".to_string(),
                    "Unsupported name attribute in payload".to_string(),
                )),
            )
                .into_response(),
            Self::NameAlreadyRegistered(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Name is already registered: {}", e),
                )),
            )
                .into_response(),
            Self::NameNotRegistered(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Name is not registered: {}", e),
                )),
            )
                .into_response(),
            Self::NameNotFound(e) => (
                StatusCode::NOT_FOUND,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Name is not found in the database: {}", e),
                )),
            )
                .into_response(),
            Self::NameByAddressNotFound => (
                StatusCode::NOT_FOUND,
                Json(new_error_response(
                    "address".to_string(),
                    "No name for address is found".into(),
                )),
            )
                .into_response(),
            Self::ExpiredTimestamp(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "timestamp".to_string(),
                    format!("Signature UNIXTIME timestamp is too old: {}", e),
                )),
            )
                .into_response(),
            Self::SignatureValidationError(e) => (
                StatusCode::UNAUTHORIZED,
                Json(new_error_response(
                    "signature".to_string(),
                    format!("Signature validation error: {}", e),
                )),
            )
                .into_response(),
            Self::NameOwnerValidationError => (
                StatusCode::UNAUTHORIZED,
                Json(new_error_response(
                    "address".to_string(),
                    "Name owner validation error".into(),
                )),
            )
                .into_response(),
            Self::SerdeJson(e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(new_error_response(
                    "".to_string(),
                    format!("Deserialization error: {}", e),
                )),
            )
                .into_response(),
            Self::RateLimited(e) => (
                StatusCode::TOO_MANY_REQUESTS,
                Json(new_error_response(
                    "rate_limit".to_string(),
                    format!("Rate limited: {}", e),
                )),
            )
                .into_response(),

            // Any other errors considering as 500
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
                .into_response(),
        };

        if response.status().is_client_error() {
            debug!("HTTP client error: {self:?}");
        }

        if response.status().is_server_error() {
            error!("HTTP server error: {self:?}");
        }

        response
    }
}

#[derive(serde::Serialize)]
pub struct ErrorReason {
    pub field: String,
    pub description: String,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub reasons: Vec<ErrorReason>,
}

pub fn new_error_response(field: String, description: String) -> ErrorResponse {
    ErrorResponse {
        status: "FAILED".to_string(),
        reasons: vec![ErrorReason { field, description }],
    }
}
