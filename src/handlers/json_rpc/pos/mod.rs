pub mod build_transactions;
pub mod check_transaction;
pub mod errors;
pub mod evm;
pub mod solana;
pub mod supported_networks;
pub mod tron;

pub use errors::{
    BuildPosTxsError,
    CheckPosTxError,
    ExecutionError,
    InternalError,
    RpcError,
    SupportedNetworksError,
    TransactionIdError,
    ValidationError,
};
use {
    crate::{
        state::AppState,
        utils::crypto::{
            disassemble_caip10_with_namespace,
            is_address_valid,
            Caip19Asset,
            Caip2ChainId,
            CaipNamespaces,
            NamespaceValidator,
        },
    },
    axum::extract::State,
    base64::{engine::general_purpose, Engine as _},
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::{convert::TryFrom, fmt::Display, str::FromStr, sync::Arc},
    strum_macros::{Display as StrumDisplay, EnumString},
    uuid::Uuid,
};

const TRANSACTION_ID_DELIMITER: &str = "|";
const TRANSACTION_ID_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, EnumString, Deserialize, Serialize)]
#[strum(serialize_all = "lowercase")]
pub enum SupportedNamespaces {
    Eip155,
    Solana,
    Tron,
}

impl NamespaceValidator for SupportedNamespaces {
    fn validate_address(&self, address: &str) -> bool {
        match self {
            SupportedNamespaces::Eip155 => is_address_valid(address, &CaipNamespaces::Eip155),
            SupportedNamespaces::Solana => is_address_valid(address, &CaipNamespaces::Solana),
            SupportedNamespaces::Tron => true,
        }
    }
}

pub trait AssetNamespaceType: FromStr + Clone + std::fmt::Debug + PartialEq {
    fn is_native(&self) -> bool;
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedNetworksResult {
    pub namespaces: Vec<SupportedNamespace>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedNamespace {
    pub name: String,
    pub methods: Vec<String>,
    pub events: Vec<String>,
    pub capabilities: Option<Value>,
    pub asset_namespaces: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTransactionParams {
    pub payment_intents: Vec<PaymentIntent>,
    pub capabilities: Option<Value>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentIntent {
    pub asset: String,
    pub amount: String,
    pub recipient: String,
    pub sender: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTransactionResult {
    pub transactions: Vec<TransactionRpc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRpc {
    pub id: String,
    pub chain_id: String,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, StrumDisplay)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTransactionParams {
    pub id: String,
    pub send_result: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTransactionResult {
    pub status: TransactionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_in: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
}
#[async_trait::async_trait]
pub trait TransactionBuilder<T: AssetNamespaceType> {
    fn namespace(&self) -> &'static str;

    async fn validate_and_build(
        &self,
        state: State<Arc<AppState>>,
        project_id: String,
        params: PaymentIntent,
    ) -> Result<TransactionRpc, BuildPosTxsError>;

    async fn build(
        &self,
        state: State<Arc<AppState>>,
        project_id: String,
        params: ValidatedPaymentIntent<T>,
    ) -> Result<TransactionRpc, BuildPosTxsError>;
}

pub struct TransactionId {
    id: String,
    chain_id: Caip2ChainId,
    version: String,
}

impl TransactionId {
    pub fn new(chain_id: &Caip2ChainId) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            chain_id: chain_id.clone(),
            version: TRANSACTION_ID_VERSION.to_string(),
        }
    }

    pub fn chain_id(&self) -> &Caip2ChainId {
        &self.chain_id
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    fn from(id: &str, chain_id: &Caip2ChainId) -> Self {
        Self {
            id: id.to_string(),
            chain_id: chain_id.clone(),
            version: TRANSACTION_ID_VERSION.to_string(),
        }
    }
}

impl Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted = [
            self.version.as_str(),
            self.chain_id.to_string().as_str(),
            &self.id,
        ]
        .join(TRANSACTION_ID_DELIMITER);
        write!(f, "{}", general_purpose::STANDARD_NO_PAD.encode(formatted))
    }
}

impl TryFrom<String> for TransactionId {
    type Error = TransactionIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for TransactionId {
    type Error = TransactionIdError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let decoded = general_purpose::STANDARD_NO_PAD
            .decode(value)
            .map_err(|_| TransactionIdError::InvalidFormat(value.to_string()))?;
        let decoded_str = String::from_utf8(decoded)
            .map_err(|_| TransactionIdError::InvalidFormat(value.to_string()))?;

        let mut parts = decoded_str.splitn(3, TRANSACTION_ID_DELIMITER);
        let version = parts
            .next()
            .ok_or_else(|| TransactionIdError::InvalidFormat(decoded_str.clone()))?;

        if version != TRANSACTION_ID_VERSION {
            return Err(TransactionIdError::InvalidFormat(decoded_str.clone()));
        }

        let chain_id_str = parts
            .next()
            .ok_or_else(|| TransactionIdError::InvalidFormat(decoded_str.clone()))?;

        let chain_id =
            Caip2ChainId::parse(chain_id_str).map_err(TransactionIdError::InvalidChainId)?;

        let id = parts
            .next()
            .ok_or_else(|| TransactionIdError::InvalidFormat(decoded_str.clone()))?;

        Ok(TransactionId::from(id, &chain_id))
    }
}

pub struct ValidatedPaymentIntent<T: AssetNamespaceType> {
    pub asset: Caip19Asset,
    pub amount: String,
    pub recipient_address: String,
    pub sender_address: String,
    pub namespace: T,
}

impl<T: AssetNamespaceType> ValidatedPaymentIntent<T> {
    pub fn validate_params(params: &PaymentIntent) -> Result<Self, BuildPosTxsError> {
        let asset = Caip19Asset::parse(&params.asset).map_err(|e| {
            BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string()))
        })?;

        let (recipient_namespace, recipient_chain_id, recipient_address) =
            disassemble_caip10_with_namespace::<SupportedNamespaces>(&params.recipient).map_err(
                |e| BuildPosTxsError::Validation(ValidationError::InvalidRecipient(e.to_string())),
            )?;

        let (sender_namespace, sender_chain_id, sender_address) =
            disassemble_caip10_with_namespace::<SupportedNamespaces>(&params.sender).map_err(
                |e| BuildPosTxsError::Validation(ValidationError::InvalidSender(e.to_string())),
            )?;

        let asset_chain_id = asset.chain_id().reference();
        let asset_namespace = asset
            .chain_id()
            .namespace()
            .parse::<SupportedNamespaces>()
            .map_err(|e| {
                BuildPosTxsError::Validation(ValidationError::InvalidAsset(e.to_string()))
            })?;

        if asset_namespace != recipient_namespace || asset_namespace != sender_namespace {
            return Err(BuildPosTxsError::Validation(ValidationError::InvalidAsset(
                "Asset namespace must match recipient and sender namespaces".to_string(),
            )));
        }

        tracing::debug!("asset_chain_id: {asset_chain_id}");
        tracing::debug!("recipient_chain_id: {recipient_chain_id}");
        tracing::debug!("sender_chain_id: {sender_chain_id}");

        if asset_chain_id != recipient_chain_id || asset_chain_id != sender_chain_id {
            return Err(BuildPosTxsError::Validation(ValidationError::InvalidAsset(
                "Asset chain ID must match recipient and sender chain IDs".to_string(),
            )));
        }

        let namespace = T::from_str(asset.asset_namespace()).map_err(|_| {
            BuildPosTxsError::Validation(ValidationError::InvalidAsset(
                asset.asset_namespace().to_string(),
            ))
        })?;

        Ok(Self {
            asset,
            amount: params.amount.clone(),
            recipient_address,
            sender_address,
            namespace,
        })
    }
}
