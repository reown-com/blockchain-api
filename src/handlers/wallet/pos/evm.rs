use {
    super::{
        BuildPosTxError, BuildTransactionParams, BuildTransactionResult, TransactionBuilder,
        TransactionRpc,
    },
    crate::{
        analytics::MessageSource,
        state::AppState,
        utils::crypto::{disassemble_caip10, Caip19Asset, CaipNamespaces, Caip2ChainId},
    },
    alloy::{
        network::TransactionBuilder as AlloyTransactionBuilder,
        primitives::{utils::parse_units, Address, U256},
        providers::{Provider, ProviderBuilder},
        rpc::types::TransactionRequest,
        sol,
    },
    async_trait::async_trait,
    axum::extract::State,
    serde::Serialize,
    std::sync::Arc,
    strum_macros::EnumString,
    tracing::debug,
    uuid::Uuid,
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
    pub value: String,
    pub data: String,
}

#[async_trait]
impl TransactionBuilder for EvmTransactionBuilder {
    fn namespace(&self) -> &'static str {
        "eip155"
    }

    async fn build(
        &self,
        _state: State<Arc<AppState>>,
        project_id: String,
        params: BuildTransactionParams,
    ) -> Result<BuildTransactionResult, BuildPosTxError> {
        let asset = Caip19Asset::parse(&params.asset)
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid Asset: {e}")))?;

        let (recipient_namespace, recipient_chain_id, recipient_address) =
            disassemble_caip10(&params.recipient)
                .map_err(|e| BuildPosTxError::Validation(format!("Invalid Recipient: {e}")))?;

        let (sender_namespace, sender_chain_id, sender_address) =
            disassemble_caip10(&params.sender)
                .map_err(|e| BuildPosTxError::Validation(format!("Invalid Sender: {e}")))?;

        let asset_chain_id = asset.chain_id().reference();
        let asset_namespace = asset
            .chain_id()
            .namespace()
            .parse::<CaipNamespaces>()
            .map_err(|e| {
                BuildPosTxError::Validation(format!("Cannot parse asset namespace: {e}"))
            })?;

        if asset_namespace != recipient_namespace || asset_namespace != sender_namespace {
            return Err(BuildPosTxError::Validation(format!(
                "Asset namespace must match recipient and sender namespaces"
            )));
        }

        debug!("asset_chain_id: {}", asset_chain_id);
        debug!("recipient_chain_id: {}", recipient_chain_id);
        debug!("sender_chain_id: {}", sender_chain_id);

        if asset_chain_id != recipient_chain_id || asset_chain_id != sender_chain_id {
            return Err(BuildPosTxError::Validation(format!(
                "Asset chain ID must match recipient and sender chain IDs"
            )));
        }

        let namespace = asset
            .asset_namespace()
            .parse::<AssetNamespace>()
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid asset namespace: {}", e)))?;

        let tx = match namespace {
            AssetNamespace::Slip44 => {
                build_native_transaction(
                    &project_id,
                    &asset.chain_id(),
                    &recipient_address,
                    &sender_address,
                    &params.amount,
                )
                .await?
            }
            AssetNamespace::Erc20 => {
                build_erc20_transaction(
                    &project_id,
                    &asset.chain_id(),
                    &recipient_address,
                    &sender_address,
                    &asset.asset_reference(),
                    &params.amount,
                )
                .await?
            }
        };

        Ok(tx)
    }
}

async fn build_native_transaction(
    project_id: &str,
    chain_id: &Caip2ChainId,
    recipient: &str,
    sender: &str,
    amount: &str,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    let provider = get_provider(chain_id, project_id)?;

    let to = recipient
        .parse::<Address>()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid recipient: {}", e)))?;

    let from = sender
        .parse::<Address>()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid sender: {}", e)))?;

    let value = parse_units(amount, "ether").map_err(|e| {
        BuildPosTxError::Validation(format!("Unable to parse amount in ether: {}", e))
    })?;

    let amount: U256 = value
        .try_into()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid amount: {}", e)))?;

    let fees = provider
        .estimate_eip1559_fees(None)
        .await
        .map_err(|e| BuildPosTxError::Validation(format!("Failed to estimate fees: {}", e)))?;

    let tx = TransactionRequest::default()
        .with_to(to)
        .with_value(amount)
        .with_gas_limit(21000)
        .with_from(from)
        .with_max_fee_per_gas(fees.max_fee_per_gas)
        .with_max_priority_fee_per_gas(fees.max_priority_fee_per_gas);
    

    debug!("native tx: {:?}", tx);
    Ok(BuildTransactionResult {
        transaction_rpc: TransactionRpc {
            method: "eth_sendTransaction".to_string(),
            params: serde_json::json!([tx]),
        },
        id: Uuid::new_v4().to_string(),
    })
}

async fn build_erc20_transaction(
    project_id: &str,
    chain_id: &Caip2ChainId,
    recipient: &str,
    sender: &str,
    asset_address: &str,
    amount: &str,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    let to = recipient
    .parse::<Address>()
    .map_err(|e| BuildPosTxError::Validation(format!("Invalid recipient: {}", e)))?;

    let from = sender
        .parse::<Address>()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid sender: {}", e)))?;

    let token_address = asset_address
        .parse::<Address>()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid asset address: {}", e)))?;

    let erc20 = ERC20Token::new(token_address, get_provider(chain_id, project_id)?);

    let decimals_call_result = erc20.decimals().call().await.map_err(|e| BuildPosTxError::Validation(format!("Failed to get decimals: {}", e)))?;
    
    let decimals = decimals_call_result._0;
    
    debug!("decimals: {:?}", decimals);
    
    let value = parse_units(amount, decimals).map_err(|e| {
        BuildPosTxError::Validation(format!("Unable to parse amount with {} decimals: {}", decimals, e))
    })?;
    
    let token_amount: U256 = value
        .try_into()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid token amount: {}", e)))?;
    

    let transfer_data = erc20.transfer(to, token_amount);
    
    let fees = get_provider(chain_id, project_id)?
        .estimate_eip1559_fees(None)
        .await
        .map_err(|e| BuildPosTxError::Validation(format!("Failed to estimate fees: {}", e)))?;

    let tx = TransactionRequest::default()
        .with_to(token_address)
        .with_value(U256::ZERO)
        .with_input(transfer_data.calldata().clone())
        .with_from(from)
        .with_max_fee_per_gas(fees.max_fee_per_gas)
        .with_max_priority_fee_per_gas(fees.max_priority_fee_per_gas);
    
    let gas_limit = get_provider(chain_id, project_id)?.estimate_gas(&tx).await.map_err(|e| BuildPosTxError::Validation(format!("Failed to estimate gas: {}", e)))?;

    let tx = tx.with_gas_limit(gas_limit);

    debug!("erc20 tx: {:?}", tx);
    
    Ok(BuildTransactionResult {
        transaction_rpc: TransactionRpc {
            method: "eth_sendTransaction".to_string(),
            params: serde_json::json!([tx]),
        },
        id: Uuid::new_v4().to_string(),
    })
}


fn get_provider(chain_id: &Caip2ChainId, project_id: &str) -> Result<impl Provider, BuildPosTxError> {
    let url = format!(
        "http://localhost:3080/v1?chainId={}&projectId={}&source={}",
        chain_id,
        project_id,
        MessageSource::WalletBuildPosTx,
    ).parse().map_err(|_| BuildPosTxError::Validation("Invalid provider URL".to_string()))?;

    Ok(ProviderBuilder::new().on_http(url))
}
