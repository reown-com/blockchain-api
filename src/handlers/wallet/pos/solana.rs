use {
    super::{
        AssetNamespaceType, BuildPosTxsError,
        CheckTransactionResult, TransactionBuilder, TransactionId, TransactionRpc,
        TransactionStatus, ValidatedPaymentIntent, PaymentIntent,
    },
    crate::{analytics::MessageSource, state::AppState, utils::crypto::Caip2ChainId},
    alloy::primitives::{utils::parse_units, U256},
    async_trait::async_trait,
    axum::extract::State,
    base64::{engine::general_purpose, Engine as _},
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        message::{v0, VersionedMessage},
        pubkey::Pubkey,
        signature::Signature,
        transaction::VersionedTransaction,
    },
    spl_associated_token_account::get_associated_token_address,
    spl_token::{instruction::transfer_checked, solana_program::program_pack::Pack, state::Mint},
    std::{str::FromStr, sync::Arc},
    strum_macros::EnumString,
    tracing::debug,
};

const SOLANA_RPC_METHOD: &str = "solana_signAndSendTransaction";
const SPL_TOKEN_2022_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const BASE_URL: &str = "https://rpc.walletconnect.org/v1";
const DEFAULT_CHECK_IN: usize = 400;

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AssetNamespace {
    Token,
    Slip44,
}

impl AssetNamespaceType for AssetNamespace {
    fn is_native(&self) -> bool {
        matches!(self, AssetNamespace::Slip44)
    }
}

pub struct SolanaTransactionBuilder;

#[async_trait]
impl TransactionBuilder<AssetNamespace> for SolanaTransactionBuilder {
    fn namespace(&self) -> &'static str {
        "solana"
    }
    async fn validate_and_build(
        &self,
        _state: State<Arc<AppState>>,
        project_id: String,
        params: PaymentIntent,
    ) -> Result<TransactionRpc, BuildPosTxsError> {
        let validated_params = ValidatedPaymentIntent::validate_params(&params)?;
        self.build(_state, project_id, validated_params).await
    }

    async fn build(
        &self,
        _state: State<Arc<AppState>>,
        project_id: String,
        params: ValidatedPaymentIntent<AssetNamespace>,
    ) -> Result<TransactionRpc, BuildPosTxsError> {

        match params.namespace {
            AssetNamespace::Token => {
                build_spl_transfer(params, &project_id).await
            }
            _ => {
                return Err(BuildPosTxsError::Validation(
                    "Unsupported asset namespace".to_string(),
                ));
            }
        }
    }
}

async fn build_spl_transfer(
    params: ValidatedPaymentIntent<AssetNamespace>,
    project_id: &str,
) -> Result<TransactionRpc, BuildPosTxsError> {
    let sender_pubkey = Pubkey::from_str(&params.sender_address)
        .map_err(|e| BuildPosTxsError::Validation(format!("Invalid sender address: {}", e)))?;

    let recipient_pubkey = Pubkey::from_str(&params.recipient_address)
        .map_err(|e| BuildPosTxsError::Validation(format!("Invalid recipient address: {}", e)))?;

    let mint_pubkey = Pubkey::from_str(params.asset.asset_reference())
        .map_err(|e| BuildPosTxsError::Validation(format!("Invalid token mint address: {}", e)))?;

    let rpc_client = create_rpc_client(params.asset.chain_id(), project_id)?;

    let (decimals, token_program_id) =
        get_token_decimals(&mint_pubkey, params.asset.chain_id(), project_id).await?;
    let amount_lamports = parse_token_amount(&params.amount, decimals)?;

    let sender_ata = get_associated_token_address(&sender_pubkey, &mint_pubkey);
    let recipient_ata = get_associated_token_address(&recipient_pubkey, &mint_pubkey);

    let transfer_instruction = transfer_checked(
        &token_program_id,
        &sender_ata,
        &mint_pubkey,
        &recipient_ata,
        &sender_pubkey,
        &[&sender_pubkey],
        amount_lamports,
        decimals,
    )
    .map_err(|e| {
        BuildPosTxsError::Validation(format!("Failed to create transfer instruction: {}", e))
    })?;

    let recent_blockhash = rpc_client
        .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
        .await
        .map_err(|e| BuildPosTxsError::Internal(format!("Failed to fetch recent blockhash: {}", e)))?
        .0;

    let instructions = vec![transfer_instruction];

    let v0_message = v0::Message::try_compile(&sender_pubkey, &instructions, &[], recent_blockhash)
        .map_err(|e| BuildPosTxsError::Internal(format!("Failed to compile v0 message: {}", e)))?;

    let message = VersionedMessage::V0(v0_message);

    let req = message.header().num_required_signatures as usize;
    let transaction = VersionedTransaction {
        signatures: vec![Signature::default(); req],
        message,
    };

    let serialized_tx = bincode::serialize(&transaction).map_err(|e| {
        BuildPosTxsError::Internal(format!("Failed to serialize transaction: {}", e))
    })?;

    let transaction_b64 = general_purpose::STANDARD.encode(serialized_tx);

   Ok(TransactionRpc {
        id: TransactionId::new(params.asset.chain_id()).to_string(),
        method: SOLANA_RPC_METHOD.to_string(),
        params: serde_json::json!({
            "transaction": transaction_b64,
            "pubkey": params.sender_address
        }),
    })
}

async fn get_token_decimals(
    mint_pubkey: &Pubkey,
    chain_id: &Caip2ChainId,
    project_id: &str,
) -> Result<(u8, Pubkey), BuildPosTxsError> {
    let rpc_client = create_rpc_client(chain_id, project_id)?;

    let mint_account = rpc_client
        .get_account_with_commitment(mint_pubkey, CommitmentConfig::confirmed())
        .await
        .map_err(|e| BuildPosTxsError::Internal(format!("Failed to fetch mint account: {}", e)))?
        .value
        .ok_or_else(|| BuildPosTxsError::Validation("Mint account not found".to_string()))?;

    let token_program_id = mint_account.owner;
    let is_spl_token = token_program_id == spl_token::id();

    let spl_token_2022_id = Pubkey::from_str(SPL_TOKEN_2022_ID)
        .map_err(|_| BuildPosTxsError::Internal("Invalid SPL Token-2022 program ID".to_string()))?;
    let is_spl_token_2022 = token_program_id == spl_token_2022_id;

    if (!is_spl_token && !is_spl_token_2022) || mint_account.data.len() < Mint::LEN {
        return Err(BuildPosTxsError::Validation(format!(
            "Invalid mint account owner: {}. Expected SPL Token program.",
            mint_account.owner
        )));
    }

    match Mint::unpack_from_slice(&mint_account.data[..Mint::LEN]) {
        Ok(mint_data) => Ok((mint_data.decimals, token_program_id)),
        Err(e) => {
            debug!("Failed to parse as SPL Token mint: {}", e);
            Err(BuildPosTxsError::Internal(format!(
                "Failed to parse as SPL Token mint: {}",
                e
            )))
        }
    }
}

fn create_rpc_client(
    chain_id: &Caip2ChainId,
    project_id: &str,
) -> Result<RpcClient, BuildPosTxsError> {
    let url = format!(
        "{BASE_URL}?chainId={chain_id}&projectId={project_id}&source={}",
        MessageSource::WalletBuildPosTx,
    );

    Ok(RpcClient::new(url))
}

fn parse_token_amount(amount: &str, decimals: u8) -> Result<u64, BuildPosTxsError> {
    let parsed_value = parse_units(amount, decimals).map_err(|e| {
        BuildPosTxsError::Validation(format!(
            "Unable to parse amount with {} decimals: {}",
            decimals, e
        ))
    })?;

    let value: U256 = parsed_value.into();

    if value > U256::from(u64::MAX) {
        return Err(BuildPosTxsError::Validation(
            "Amount too large for u64".to_string(),
        ));
    }

    Ok(value.to::<u64>())
}

pub async fn get_transaction_status(
    _state: State<Arc<AppState>>,
    project_id: &str,
    signature: &str,
    chain_id: &Caip2ChainId,
) -> Result<TransactionStatus, BuildPosTxsError> {
    let parsed_signature = Signature::from_str(signature)
        .map_err(|e| BuildPosTxsError::Validation(format!("Invalid signature: {}", e)))?;

    let rpc_client = create_rpc_client(chain_id, project_id)?;

    let response = rpc_client
        .get_signature_statuses_with_history(&[parsed_signature])
        .await
        .map_err(|e| BuildPosTxsError::Internal(format!("Failed to get signature status: {}", e)))?;

    match response.value.first() {
        Some(Some(status)) => {
            if status.err.is_some() {
                Ok(TransactionStatus::Failed)
            } else {
                Ok(TransactionStatus::Confirmed)
            }
        }
        Some(None) => Ok(TransactionStatus::Pending),
        None => Ok(TransactionStatus::Pending),
    }
}

pub async fn check_transaction(
    state: State<Arc<AppState>>,
    project_id: &str,
    signature: &str,
    chain_id: &Caip2ChainId,
) -> Result<CheckTransactionResult, BuildPosTxsError> {
    let status = get_transaction_status(state, project_id, signature, chain_id).await?;

    match status {
        TransactionStatus::Pending => Ok(CheckTransactionResult {
            status,
            check_in: Some(DEFAULT_CHECK_IN),
        }),
        _ => Ok(CheckTransactionResult {
            status,
            check_in: None,
        }),
    }
}
