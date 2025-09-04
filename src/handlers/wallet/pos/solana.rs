use {
    super::{
        AssetNamespaceType, BuildPosTxError, BuildTransactionParams, BuildTransactionResult,
        TransactionBuilder, TransactionId, TransactionRpc, TransactionStatus,
        ValidatedTransactionParams,
    },
    crate::{analytics::MessageSource, state::AppState, utils::crypto::Caip2ChainId},
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
impl TransactionBuilder for SolanaTransactionBuilder {
    fn namespace(&self) -> &'static str {
        "solana"
    }

    async fn build(
        &self,
        _state: State<Arc<AppState>>,
        project_id: String,
        params: BuildTransactionParams,
    ) -> Result<BuildTransactionResult, BuildPosTxError> {
        let validated_params: ValidatedTransactionParams<AssetNamespace> =
            ValidatedTransactionParams::validate_params(&params)?;

        match validated_params.namespace {
            AssetNamespace::Token => {
                build_spl_transfer(validated_params, &params.amount, &project_id).await
            }
            _ => {
                return Err(BuildPosTxError::Validation(
                    "Unsupported asset namespace".to_string(),
                ));
            }
        }
    }
}

async fn build_spl_transfer(
    params: ValidatedTransactionParams<AssetNamespace>,
    amount: &str,
    project_id: &str,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    let sender_pubkey = Pubkey::from_str(&params.sender_address)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid sender address: {}", e)))?;

    let recipient_pubkey = Pubkey::from_str(&params.recipient_address)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid recipient address: {}", e)))?;

    let mint_pubkey = Pubkey::from_str(params.asset.asset_reference())
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid token mint address: {}", e)))?;

    let rpc_client = create_rpc_client(params.asset.chain_id(), project_id)?;

    let (decimals, token_program_id) =
        get_token_decimals(&mint_pubkey, params.asset.chain_id(), project_id).await?;
    let amount_lamports = parse_token_amount(amount, decimals)?;

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
        BuildPosTxError::Validation(format!("Failed to create transfer instruction: {}", e))
    })?;

    let recent_blockhash = rpc_client
        .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to fetch recent blockhash: {}", e)))?
        .0;

    let instructions = vec![transfer_instruction];

    let v0_message = v0::Message::try_compile(&sender_pubkey, &instructions, &[], recent_blockhash)
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to compile v0 message: {}", e)))?;

    let message = VersionedMessage::V0(v0_message);

    let req = message.header().num_required_signatures as usize;
    let transaction = VersionedTransaction {
        signatures: vec![Signature::default(); req],
        message,
    };

    let serialized_tx = bincode::serialize(&transaction).map_err(|e| {
        BuildPosTxError::Internal(format!("Failed to serialize transaction: {}", e))
    })?;

    let transaction_b64 = general_purpose::STANDARD.encode(serialized_tx);

    let transaction_rpc = TransactionRpc {
        method: SOLANA_RPC_METHOD.to_string(),
        params: serde_json::json!({
            "transaction": transaction_b64,
            "pubkey": params.sender_address
        }),
    };

    let id = TransactionId::new(params.asset.chain_id()).to_string();

    Ok(BuildTransactionResult {
        transaction_rpc,
        id,
    })
}

async fn get_token_decimals(
    mint_pubkey: &Pubkey,
    chain_id: &Caip2ChainId,
    project_id: &str,
) -> Result<(u8, Pubkey), BuildPosTxError> {
    let rpc_client = create_rpc_client(chain_id, project_id)?;

    let mint_account = rpc_client
        .get_account_with_commitment(mint_pubkey, CommitmentConfig::confirmed())
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to fetch mint account: {}", e)))?
        .value
        .ok_or_else(|| BuildPosTxError::Validation("Mint account not found".to_string()))?;

    let token_program_id = mint_account.owner;
    let is_spl_token = token_program_id == spl_token::id();

    let spl_token_2022_id = Pubkey::from_str(SPL_TOKEN_2022_ID)
        .map_err(|_| BuildPosTxError::Internal("Invalid SPL Token-2022 program ID".to_string()))?;
    let is_spl_token_2022 = token_program_id == spl_token_2022_id;

    if !is_spl_token && !is_spl_token_2022 && mint_account.data.len() < Mint::LEN {
        return Err(BuildPosTxError::Validation(format!(
            "Invalid mint account owner: {}. Expected SPL Token program.",
            mint_account.owner
        )));
    }

    match Mint::unpack_from_slice(&mint_account.data[..Mint::LEN]) {
        Ok(mint_data) => Ok((mint_data.decimals, token_program_id)),
        Err(e) => {
            debug!("Failed to parse as SPL Token mint: {}", e);
            Err(BuildPosTxError::Internal(format!(
                "Failed to parse as SPL Token mint: {}",
                e
            )))
        }
    }
}

fn create_rpc_client(
    chain_id: &Caip2ChainId,
    project_id: &str,
) -> Result<RpcClient, BuildPosTxError> {
    let url = format!(
        "{BASE_URL}?chainId={chain_id}&projectId={project_id}&source={}",
        MessageSource::WalletBuildPosTx,
    );

    Ok(RpcClient::new(url))
}

fn parse_token_amount(amount: &str, decimals: u8) -> Result<u64, BuildPosTxError> {
    let parsed_amount: f64 = amount
        .parse()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid amount format: {}", e)))?;

    let decimal_multiplier = 10_u64.pow(decimals as u32) as f64;
    let lamports = (parsed_amount * decimal_multiplier) as u64;

    Ok(lamports)
}

pub async fn get_transaction_status(
    _state: State<Arc<AppState>>,
    project_id: &str,
    signature: &str,
    chain_id: &Caip2ChainId,
) -> Result<TransactionStatus, BuildPosTxError> {
    let parsed_signature = Signature::from_str(signature)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid signature: {}", e)))?;

    let rpc_client = create_rpc_client(chain_id, project_id)?;

    let response = rpc_client
        .get_signature_statuses_with_history(&[parsed_signature])
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to get signature status: {}", e)))?;

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
