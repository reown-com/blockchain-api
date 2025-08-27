use {
    super::{BuildPosTxError, BuildTransactionParams, BuildTransactionResult, TransactionBuilder, TransactionRpc},
    crate::state::AppState,
    alloy::{
        sol,
    },
    axum::extract::State,
    serde::{Serialize},
    async_trait::async_trait,
    std::{sync::Arc},
    strum_macros::EnumString,
    crate::utils::crypto::{disassemble_caip10, Caip19Asset},
};

sol! {
    #[sol(rpc)]
    interface ERC20Token {
        function transfer(address to, uint256 value) external returns (bool);
        function decimals() external view returns (uint8);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EvmTransactionType {
    Erc20,
    Slip44,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AssetNamespace {
    Erc20,
    Slip44,
}


pub struct EvmTransactionBuilder;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EthTx {
    pub to: String,
    pub from: Option<String>,
    pub value: String,
    pub data: String,
}


#[async_trait]
impl TransactionBuilder for EvmTransactionBuilder {
    fn namespace(&self) -> &'static str { "eip155" }

    async fn build(
        &self,
        _state: State<Arc<AppState>>,
        project_id: String,
        params: BuildTransactionParams,
    ) -> Result<BuildTransactionResult, BuildPosTxError> {
        let asset = Caip19Asset::parse(&params.asset)
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid Asset: {e}")))?;

        let (recipient_namespace, recipient_chain_id, recipient_address) = disassemble_caip10(&params.recipient)
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid Recipient: {e}")))?;

        let (sender_namespace, sender_chain_id, sender_address) = disassemble_caip10(&params.sender)
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid Sender: {e}")))?;

        let (asset_namespace, asset_chain_id, asset_address) = disassemble_caip10(&params.asset)
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid Asset: {e}")))?;


        if asset_namespace != recipient_namespace || asset_namespace != sender_namespace {
            return Err(BuildPosTxError::Validation(format!("Asset namespace must match recipient and sender namespaces")));
        }

        if asset_chain_id != recipient_chain_id || asset_chain_id != sender_chain_id {
            return Err(BuildPosTxError::Validation(format!("Asset chain ID must match recipient and sender chain IDs")));
        }

        let namespace = asset.asset_namespace().parse::<AssetNamespace>()
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid asset namespace: {}", e)))?;

        let tx = match namespace {
            AssetNamespace::Erc20 => {
                Ok(BuildTransactionResult {
                    transaction_rpc: TransactionRpc {
                        method: "eth_sendTransaction".to_string(),
                        params: serde_json::json!([tx]),
                    },
                    id: "1".to_string(),
                })
            },
            AssetNamespace::Slip44 => {
                Ok(BuildTransactionResult {
                    transaction_rpc: TransactionRpc {
                        method: "eth_sendTransaction".to_string(),
                        params: serde_json::json!([tx]),
                    },
                    id: "1".to_string(),
                })
            },
        };
        
    }
}




