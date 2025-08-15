use {
    crate::{
        handlers::{
            chain_agnostic::route::RouteSolanaError, sessions::get::InternalGetSessionContextError,
        },
        project::ProjectDataError,
        storage::error::StorageError,
        utils::crypto::{CaipNamespaces, CryptoUitlsError},
    },
    axum::{response::IntoResponse, Json},
    cerberus::registry::RegistryError,
    hyper::StatusCode,
    tracing::log::error,
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
    #[error("Hyper util error: {0}")]
    HyperUtilError(#[from] hyper_util::client::legacy::Error),

    #[error("Proxy timeout error: {0}")]
    ProxyTimeoutError(tokio::time::error::Elapsed),

    #[error("Request::builder() failed: {0}")]
    RequestBuilderError(#[from] hyper::http::Error),

    #[error("Specified chain is not supported by any of the providers: {0}")]
    UnsupportedChain(String),

    #[error("Specified currency is not supported: {0}")]
    UnsupportedCurrency(String),

    #[error("Requested chain provider is temporarily unavailable: {0}")]
    ChainTemporarilyUnavailable(String),

    #[error("Invalid chainId format for the requested namespace: {0}")]
    InvalidChainIdFormat(String),

    #[error("Specified provider is not supported: {0}")]
    UnsupportedProvider(String),

    #[error("Specified bundler is not supported: {0}")]
    UnsupportedBundler(String),

    #[error("Failed to reach the identity provider: {0}")]
    IdentityProviderError(String),

    #[error("Failed to reach the transaction provider")]
    TransactionProviderError,

    #[error("Failed to reach the portfolio provider")]
    PortfolioProviderError,

    #[error("Failed to reach the balance provider")]
    BalanceProviderError,

    #[error("Requested balance provider for the namespace is temporarily unavailable: {0}")]
    BalanceTemporarilyUnavailable(String),

    #[error("Failed to reach the fungible price provider: {0}")]
    FungiblePriceProviderError(String),

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

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Only WebSocket connections are supported for GET method on this endpoint")]
    WebSocketConnectionExpected,

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

    #[error("Invalid conversion parameter with code: {0} and description: {1}")]
    ConversionInvalidParameterWithCode(String, String),

    #[error("Conversion provider internal error: {0}")]
    ConversionProviderInternalError(String),

    // Profile names errors
    #[error("Name is already registered: {0}")]
    NameAlreadyRegistered(String),

    #[error("Name is not registered: {0}")]
    NameNotRegistered(String),

    #[error("Name registeration error: {0}")]
    NameRegistrationError(String),

    #[error("Name is not found: {0}")]
    NameNotFound(String),

    #[error("No name is found for address")]
    NameByAddressNotFound,

    #[error("Internal name resolver error")]
    InternalNameResolverError,

    #[error("Invalid name format: {0}")]
    InvalidNameFormat(String),

    #[error("Invalid name length: {0}")]
    InvalidNameLength(String),

    #[error("Name is not in the allowed zones: {0}")]
    InvalidNameZone(String),

    #[error("Invalid value: {0}")]
    InvalidValue(String),

    #[error("Unsupported coin type: {0}")]
    UnsupportedCoinType(u32),

    #[error("Unsupported namespace: {0}")]
    UnsupportedNamespace(CaipNamespaces),

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

    #[error("IRN client is not configured")]
    IrnNotConfigured,

    #[error("Internal permissions get context error: {0}")]
    InternalGetSessionContextError(InternalGetSessionContextError),

    #[error("Wrong Base64 format: {0}")]
    WrongBase64Format(String),

    #[error("Wrong Hex format: {0}")]
    WrongHexFormat(String),

    #[error("Key format error: {0}")]
    KeyFormatError(String),

    #[error("Signature format error: {0}")]
    SignatureFormatError(String),

    #[error("Pkcs8 error: {0}")]
    Pkcs8Error(#[from] ethers::core::k256::pkcs8::Error),

    #[error("Permission for PCI is not found: {0} {1}")]
    PermissionNotFound(String, String),

    #[error("Permission context was not updated yet: {0}")]
    PermissionContextNotUpdated(String),

    #[error("Permission is revoked: {0}")]
    RevokedPermission(String),

    #[error("Permission is expired: {0}")]
    PermissionExpired(String),

    #[error("Permissions set is empty")]
    CoSignerEmptyPermissions,

    #[error("Cosigner permission denied: {0}")]
    CosignerPermissionDenied(String),

    #[error("Cosigner unsupported permission: {0}")]
    CosignerUnsupportedPermission(String),

    #[error("ABI decoding error: {0}")]
    AbiDecodingError(String),

    #[error("Orchestration ID is not found: {0}")]
    OrchestrationIdNotFound(String),

    #[error("Bridging final amount is less then expected")]
    BridgingFinalAmountLess,

    #[error("Simulation provider unavailable")]
    SimulationProviderUnavailable,

    #[error("Simulation failed: {0}")]
    SimulationFailed(String),

    #[error("Route solana: {0}")]
    RouteSolana(#[from] RouteSolanaError),

    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Unsupported bundler name (URL parse error): {0}")]
    UnsupportedBundlerNameUrlParseError(url::ParseError),

    #[error("Unsupported bundler name: {0}")]
    UnsupportedBundlerName(String),
}

impl IntoResponse for RpcError {
    fn into_response(self) -> axum::response::Response {
        let response =  match &self {
            Self::WebSocketError(err) => (StatusCode::GONE, err.to_string()).into_response(),
            Self::UnsupportedChain(chain_id) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "chainId".to_string(),
                    format!("We don't support the chainId you provided: {chain_id}. See the list of supported chains here: https://docs.reown.com/cloud/blockchain-api#supported-chains"),
                )),
            )
                .into_response(),
            Self::UnsupportedCurrency(error_message) => (
                    StatusCode::BAD_REQUEST,
                    Json(new_error_response(
                        "currency".to_string(),
                        format!("Unsupported currency: {error_message}."),
                    )),
                )
                    .into_response(),
            Self::UnsupportedBundlerName(error_message) => (
                    StatusCode::BAD_REQUEST,
                    Json(new_error_response(
                        "bundler_name".to_string(),
                        format!("Unsupported bundler name: {error_message}."),
                    )),
                )
                    .into_response(),
            Self::UnsupportedBundlerNameUrlParseError(error_message) => (
                    StatusCode::BAD_REQUEST,
                    Json(new_error_response(
                        "bundler_name".to_string(),
                        format!("Unsupported bundler name: {error_message}."),
                    )),
                )
                    .into_response(),
            Self::CryptoUitlsError(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "".to_string(),
                    format!("Crypto utils error: {e}"),
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
            Self::BalanceTemporarilyUnavailable(namespace) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "chainId".to_string(),
                    format!("Requested namespace {namespace} balance provider is temporarily unavailable"),
                )),
            )
                .into_response(),
            Self::InvalidChainIdFormat(chain_id) => (
                    StatusCode::BAD_REQUEST,
                    Json(new_error_response(
                        "chainId".to_string(),
                        format!("Requested {chain_id} has invalid format for the requested namespace"),
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
            Self::UnsupportedBundler(bundler) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "bundler".to_string(),
                    format!("Bundler {bundler} is not supported"),
                )),
            )
                .into_response(),
            Self::IdentityProviderError(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "".to_string(),
                    format!("We failed to reach the identity provider with an error: {e}"),
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
            Self::WebSocketConnectionExpected => (
                StatusCode::UPGRADE_REQUIRED,
                Json(new_error_response(
                    "".to_string(),
                    "Only WebSocket connections are supported for GET method on this endpoint".to_string(),
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
                    format!("Invalid parameter: {e}"),
                )),
            )
                .into_response(),

            // Profile names errors
            Self::InvalidNameFormat(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Invalid name format: {e}"),
                )),
            )
                .into_response(),
            Self::InvalidNameLength(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Invalid name length: {e}"),
                )),
            )
                .into_response(),
            Self::InvalidNameZone(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Name is not in the allowed zones: {e}"),
                )),
            )
                .into_response(),
            Self::ConversionInvalidParameter(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(new_error_response(
                        "".to_string(),
                        format!("Conversion parameter error: {e}"),
                    )),
                )
                    .into_response(),
            Self::ConversionInvalidParameterWithCode(code, message) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response_with_code(
                    code.to_string(),
                    format!("Conversion parameter error: {message}"),
                )),
            )
                .into_response(),
            Self::UnsupportedCoinType(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "coin_type".to_string(),
                    format!("Unsupported coin type: {e}"),
                )),
            )
                .into_response(),
                Self::UnsupportedNamespace(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(new_error_response(
                        "address".to_string(),
                        format!("Unsupported namespace: {e}"),
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
                    format!("Name is already registered: {e}"),
                )),
            )
                .into_response(),
            Self::NameNotRegistered(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Name is not registered: {e}"),
                )),
            )
                .into_response(),
            Self::NameNotFound(e) => (
                StatusCode::NOT_FOUND,
                Json(new_error_response(
                    "name".to_string(),
                    format!("Name is not found in the database: {e}"),
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
                    format!("Signature UNIXTIME timestamp is too old: {e}"),
                )),
            )
                .into_response(),
            Self::SignatureValidationError(e) => (
                StatusCode::UNAUTHORIZED,
                Json(new_error_response(
                    "signature".to_string(),
                    format!("Signature validation error: {e}"),
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
                    format!("Deserialization error: {e}"),
                )),
            )
                .into_response(),
            Self::RateLimited(e) => (
                StatusCode::TOO_MANY_REQUESTS,
                Json(new_error_response(
                    "rate_limited".to_string(),
                    format!("Requests per second limit exceeded: {e}"),
                )),
            )
                .into_response(),
            Self::PermissionNotFound(address, pci) => {
                // TODO: Remove this debug log
                print!(
                    "Permission not found with PCI: {pci:?} and address: {address:?}"
                );
                (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "pci".to_string(),
                    format!("Permission for PCI is not found: {pci}"),
                )),
            )
                .into_response()
            },
            Self::RevokedPermission(pci) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "pci".to_string(),
                    format!("Permission is revoked: {pci}"),
                )),
            )
                .into_response(),
            Self::PermissionExpired(pci) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "pci".to_string(),
                    format!("Permission is expired: {pci}"),
                )),
            )
                .into_response(),
            Self::WrongBase64Format(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "".to_string(),
                    format!("Wrong Base64 format: {e}"),
                )),
            )
                .into_response(),
            Self::KeyFormatError(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "key".to_string(),
                    format!("Invalid key format: {e}"),
                )),
            )
                .into_response(),
            Self::SignatureFormatError(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "signature".to_string(),
                    format!("Invalid signature format: {e}"),
                )),
            )
                .into_response(),
            Self::CoSignerEmptyPermissions => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "".to_string(),
                    "Permissions set is empty".to_string(),
                )),
            )
                .into_response(),
            Self::AbiDecodingError(e) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "calldata".to_string(),
                    format!("ABI signature decoding error: {e}"),
                )),
            )
                .into_response(),
            Self::TransactionProviderError => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "".to_string(),
                    "Transaction provider is temporarily unavailable".to_string(),
                )),
            )
                .into_response(),
            Self::OnRampProviderError => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(new_error_response(
                        "".to_string(),
                        "OnRamp provider is temporarily unavailable".to_string(),
                    )),
                )
                    .into_response(),
            Self::PortfolioProviderError => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "".to_string(),
                    "Portfolio provider is temporarily unavailable".to_string(),
                )),
            )
                .into_response(),
            Self::BalanceProviderError => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "".to_string(),
                    "Balance provider is temporarily unavailable".to_string(),
                )),
            )
                .into_response(),
            Self::FungiblePriceProviderError(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "".to_string(),
                    format!("Fungibles price provider is temporarily unavailable: {e}"),
                )),
            )
                .into_response(),
            Self::ConversionProviderError => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "".to_string(),
                    "Convertion provider is temporarily unavailable".to_string(),
                )),
            )
                .into_response(),
            Self::SimulationProviderUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(new_error_response(
                    "".to_string(),
                    "Simulation provider is temporarily unavailable".to_string(),
                )),
            )
                .into_response(),
            Self::CosignerPermissionDenied(e) => (
                    StatusCode::UNAUTHORIZED,
                    Json(new_error_response(
                        "".to_string(),
                        format!("Cosigner permission denied: {e}"),
                    )),
                )
                    .into_response(),
            Self::CosignerUnsupportedPermission(e) => (
                StatusCode::UNAUTHORIZED,
                Json(new_error_response(
                    "".to_string(),
                    format!("Unsupported permission in CoSigner: {e}"),
                )),
            )
                .into_response(),
            Self::OrchestrationIdNotFound(id) => (
                StatusCode::BAD_REQUEST,
                Json(new_error_response(
                    "orchestrationId".to_string(),
                    format!("Orchestration ID is not found: {id}"),
                )),
            )
                .into_response(),
            Self::RouteSolana(e) => e.into_response(),
            // Any other errors considering as 500
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
                .into_response(),
        };

        // Log the server errors response status based on the status code
        match response.status() {
            StatusCode::INTERNAL_SERVER_ERROR => {
                error!("HTTP internal server error: {self:?}");
            }
            status if status.is_server_error() => {
                error!("HTTP server error: {self:?}");
            }
            _ => {}
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

#[derive(serde::Serialize)]
pub struct ErrorResponseWithCode {
    pub code: String,
    pub message: String,
}

pub fn new_error_response_with_code(code: String, message: String) -> ErrorResponseWithCode {
    ErrorResponseWithCode { code, message }
}
