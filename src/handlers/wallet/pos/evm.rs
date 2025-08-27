use {
    super::{
        BuildPosTxError, BuildTransactionParams, BuildTransactionResult, TransactionBuilder,
        TransactionRpc,
    },
    crate::{
        analytics::MessageSource,
        state::AppState,
        utils::crypto::{disassemble_caip10, Caip19Asset},
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

        let (asset_namespace, asset_chain_id, asset_address) = disassemble_caip10(&params.asset)
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid Asset: {e}")))?;

        if asset_namespace != recipient_namespace || asset_namespace != sender_namespace {
            return Err(BuildPosTxError::Validation(format!(
                "Asset namespace must match recipient and sender namespaces"
            )));
        }

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
                    &asset_chain_id,
                    &recipient_address,
                    &sender_address,
                    &params.amount,
                )
                .await?
            }
            AssetNamespace::Erc20 => {
                build_erc20_transaction(
                    &project_id,
                    &asset_chain_id,
                    &recipient_address,
                    &asset_address,
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
    chain_id: &str,
    recipient: &str,
    sender: &str,
    amount: &str,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    let provider = ProviderBuilder::new().on_http(
        format!(
            "https://rpc.walletconnect.org/v1?chainId={}&projectId={}&source={}",
            chain_id,
            project_id,
            MessageSource::WalletBuildPosTx,
        )
        .parse()
        .unwrap(),
    );
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

    Ok(BuildTransactionResult {
        transaction_rpc: TransactionRpc {
            method: "eth_sendTransaction".to_string(),
            params: serde_json::json!(tx),
        },
        id: "1".to_string(),
    })
}

async fn build_erc20_transaction(
    project_id: &str,
    chain_id: &str,
    recipient_address: &str,
    asset_address: &str,
    amount: &str,
) -> Result<BuildTransactionResult, BuildPosTxError> {
    // let provider = ProviderBuilder::default().on_http(
    //     format!(
    //         "https://rpc.walletconnect.org/v1?chainId={}&projectId={}&source={}",
    //         chain_id,
    //         project_id,
    //         MessageSource::WalletBuildPosTx,
    //     )
    //     .parse()
    //     .unwrap(),
    // );
    Ok(BuildTransactionResult {
        transaction_rpc: TransactionRpc {
            method: "eth_sendTransaction".to_string(),
            params: serde_json::json!([]),
        },
        id: "1".to_string(),
    })
}
