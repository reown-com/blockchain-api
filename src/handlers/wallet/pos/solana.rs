use {
    super::{
        AssetNamespaceType, BuildPosTxError, BuildTransactionParams, BuildTransactionResult, 
        TransactionBuilder, TransactionId, TransactionRpc, ValidatedTransactionParams,
    },
    crate::{
        state::AppState,
        analytics::MessageSource,
    },
    async_trait::async_trait,
    axum::extract::State,
    base64::{engine::general_purpose, Engine as _},
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        message::Message,
        pubkey::Pubkey,
        transaction::Transaction,
    },
    spl_associated_token_account::get_associated_token_address,
    spl_token::{
        instruction::transfer_checked, 
        state::Mint,
        solana_program::program_pack::Pack,
    },
    std::{str::FromStr, sync::Arc},
    strum_macros::{EnumString},
};

const SOLANA_RPC_METHOD: &str = "solana_signAndSendTransaction";
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

    async fn build(&self, _state: State<Arc<AppState>>, project_id: String, params: BuildTransactionParams) -> Result<BuildTransactionResult, BuildPosTxError> {
        let validated_params: ValidatedTransactionParams<AssetNamespace> = 
            ValidatedTransactionParams::validate_params(&params)?;
        
        match validated_params.namespace {
            AssetNamespace::Token => {
                build_spl_transfer(validated_params, &params.amount, &project_id).await
            },
            _ => {
                return Err(BuildPosTxError::Validation("Unsupported asset namespace".to_string()));
            }
        }
    }
}


async fn build_spl_transfer(params: ValidatedTransactionParams<AssetNamespace>, amount: &str, project_id: &str) -> Result<BuildTransactionResult, BuildPosTxError> {
    let sender_pubkey = Pubkey::from_str(&params.sender_address)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid sender address: {}", e)))?;
    
    let recipient_pubkey = Pubkey::from_str(&params.recipient_address)
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid recipient address: {}", e)))?;
    
    let mint_pubkey = Pubkey::from_str(params.asset.asset_reference())
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid token mint address: {}", e)))?;
    
    let sender_ata = get_associated_token_address(&sender_pubkey, &mint_pubkey);
    let recipient_ata = get_associated_token_address(&recipient_pubkey, &mint_pubkey);
    
    // Fetch token decimals from the mint account
    let decimals = get_token_decimals(&mint_pubkey, params.asset.chain_id(), project_id).await?;
    let amount_lamports = parse_token_amount(amount, decimals)?;
    
    let transfer_instruction = transfer_checked(
        &spl_token::id(),
        &sender_ata,
        &mint_pubkey,
        &recipient_ata,
        &sender_pubkey,
        &[&sender_pubkey],
        amount_lamports,
        decimals,
    ).map_err(|e| BuildPosTxError::Validation(format!("Failed to create transfer instruction: {}", e)))?;
    
    let instructions = vec![transfer_instruction];
    let message = Message::new(&instructions, Some(&sender_pubkey));
    let transaction = Transaction::new_unsigned(message);
    
    let serialized_tx = bincode::serialize(&transaction)
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to serialize transaction: {}", e)))?;
    
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
    chain_id: &crate::utils::crypto::Caip2ChainId,
    project_id: &str
) -> Result<u8, BuildPosTxError> {
    let rpc_client = create_rpc_client(chain_id, project_id)?;
    
    let mint_account = rpc_client
        .get_account_with_commitment(mint_pubkey, CommitmentConfig::confirmed())
        .await
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to fetch mint account: {}", e)))?
        .value
        .ok_or_else(|| BuildPosTxError::Validation("Mint account not found".to_string()))?;
    
    
    let mint_data = Mint::unpack_from_slice(&mint_account.data)
        .map_err(|e| BuildPosTxError::Internal(format!("Failed to parse mint data: {}", e)))?;
    
    Ok(mint_data.decimals)
}

fn create_rpc_client(
    chain_id: &crate::utils::crypto::Caip2ChainId,
    project_id: &str,
) -> Result<RpcClient, BuildPosTxError> {
    let url = format!(
        "{BASE_URL}?chainId={chain_id}&projectId={project_id}&source={}",
        MessageSource::WalletBuildPosTx,
    );
    
    Ok(RpcClient::new(url))
}

fn parse_token_amount(amount: &str, decimals: u8) -> Result<u64, BuildPosTxError> {
    let parsed_amount: f64 = amount.parse()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid amount format: {}", e)))?;
    
    let decimal_multiplier = 10_u64.pow(decimals as u32) as f64;
    let lamports = (parsed_amount * decimal_multiplier) as u64;
    
    Ok(lamports)
}

