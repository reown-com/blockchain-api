use {
    super::{
        AssetNamespaceType, BuildPosTxError, BuildTransactionParams, BuildTransactionResult,
        TransactionBuilder, TransactionId, TransactionRpc, TransactionStatus,
        ValidatedTransactionParams,
    },
    crate::{analytics::MessageSource, state::AppState, utils::crypto::Caip2ChainId},
    alloy::{
        primitives::{utils::parse_units, Address, TxHash, U256},
        providers::{Provider, ProviderBuilder},
        rpc::types::TransactionRequest,
        sol,
    },
    async_trait::async_trait,
    axum::extract::State,
    std::sync::Arc,
    strum_macros::EnumString,
    tracing::debug,
};

const NATIVE_GAS_LIMIT: u64 = 21_000;
const ETH_SEND_TRANSACTION_METHOD: &str = "eth_sendTransaction";
const BASE_URL: &str = "https://rpc.walletconnect.org/v1";

sol! {
    #[sol(rpc)]
    interface ERC20Token {
        function transfer(address to, uint256 value) external returns (bool);
        function decimals() external view returns (uint8);
    }
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AssetNamespace {
    Erc20,
    Slip44,
}

impl AssetNamespaceType for AssetNamespace {
    fn is_native(&self) -> bool {
        matches!(self, AssetNamespace::Slip44)
    }
}

pub struct EvmTransactionBuilder;

#[derive(Debug)]
struct EvmTxBuilder {
    to: Address,
    from: Address,
    tx_request: TransactionRequest,
    project_id: String,
    chain_id: Caip2ChainId,
}

impl EvmTxBuilder {
    fn new(
        project_id: &str,
        chain_id: &Caip2ChainId,
        recipient: &str,
        sender: &str,
    ) -> Result<Self, BuildPosTxError> {
        let to = recipient
            .parse::<Address>()
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid recipient: {}", e)))?;

        let from = sender
            .parse::<Address>()
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid sender: {}", e)))?;

        Ok(Self {
            to,
            from,
            tx_request: TransactionRequest::default(),
            project_id: project_id.to_string(),
            chain_id: chain_id.clone(),
        })
    }

    async fn with_native_transfer(mut self, amount: &str) -> Result<Self, BuildPosTxError> {
        let wei_value = parse_ether_amount(amount)?;

        self.tx_request = self.tx_request.to(self.to).value(wei_value).from(self.from);

        Ok(self)
    }

    async fn with_erc20_transfer(
        mut self,
        asset_address: &str,
        amount: &str,
    ) -> Result<Self, BuildPosTxError> {
        let token_address = asset_address
            .parse::<Address>()
            .map_err(|e| BuildPosTxError::Validation(format!("Invalid asset address: {}", e)))?;
        let provider = get_provider(&self.chain_id, &self.project_id)?;

        let token_amount = get_erc20_transfer_amount(&provider, token_address, amount).await?;
        let transfer_calldata =
            create_erc20_transfer_calldata(token_address, &provider, self.to, token_amount).await?;

        self.tx_request = self
            .tx_request
            .to(token_address)
            .value(U256::ZERO)
            .input(transfer_calldata)
            .from(self.from);

        self.tx_request.input.data = self.tx_request.input.input.clone();

        Ok(self)
    }

    async fn finalize(mut self) -> Result<BuildTransactionResult, BuildPosTxError> {
        let provider = get_provider(&self.chain_id, &self.project_id)?;

        let fees = provider
            .estimate_eip1559_fees(None)
            .await
            .map_err(|e| BuildPosTxError::Validation(format!("Failed to estimate fees: {e}")))?;

        self.tx_request = self
            .tx_request
            .max_fee_per_gas(fees.max_fee_per_gas)
            .max_priority_fee_per_gas(fees.max_priority_fee_per_gas);

        let gas_limit = if has_transaction_data(&self.tx_request) {
            provider
                .estimate_gas(&self.tx_request)
                .await
                .map_err(|e| BuildPosTxError::Validation(format!("Failed to estimate gas: {e}")))?
        } else {
            NATIVE_GAS_LIMIT
        };

        self.tx_request = self.tx_request.gas_limit(gas_limit);
        debug!("finalized tx: {:?}", self.tx_request);

        Ok(BuildTransactionResult {
            transaction_rpc: TransactionRpc {
                method: ETH_SEND_TRANSACTION_METHOD.to_string(),
                params: serde_json::json!([self.tx_request]),
            },
            id: TransactionId::new(&self.chain_id).to_string(),
        })
    }
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
        let validated_params: ValidatedTransactionParams<AssetNamespace> =
            ValidatedTransactionParams::validate_params(&params)?;

        let builder = EvmTxBuilder::new(
            &project_id,
            validated_params.asset.chain_id(),
            &validated_params.recipient_address,
            &validated_params.sender_address,
        )?;

        let tx = match validated_params.namespace {
            AssetNamespace::Slip44 => {
                builder
                    .with_native_transfer(&params.amount)
                    .await?
                    .finalize()
                    .await?
            }
            AssetNamespace::Erc20 => {
                builder
                    .with_erc20_transfer(validated_params.asset.asset_reference(), &params.amount)
                    .await?
                    .finalize()
                    .await?
            }
        };

        Ok(tx)
    }
}

fn parse_ether_amount(amount: &str) -> Result<U256, BuildPosTxError> {
    let value = parse_units(amount, "ether").map_err(|e| {
        BuildPosTxError::Validation(format!("Unable to parse amount in ether: {e}"))
    })?;

    Ok(value.into())
}

fn has_transaction_data(tx_request: &TransactionRequest) -> bool {
    tx_request.input.data.is_some() || tx_request.input.input.is_some()
}

async fn get_erc20_transfer_amount(
    provider: &impl Provider,
    token_address: Address,
    amount: &str,
) -> Result<U256, BuildPosTxError> {
    let erc20 = ERC20Token::new(token_address, provider);

    let decimals = erc20
        .decimals()
        .call()
        .await
        .map_err(|e| BuildPosTxError::Validation(format!("Failed to get decimals: {e}")))?
        ._0;

    debug!("decimals: {decimals}");

    let value = parse_units(amount, decimals).map_err(|e| {
        BuildPosTxError::Validation(format!(
            "Unable to parse amount with {decimals} decimals: {e}"
        ))
    })?;

    Ok(value.into())
}

async fn create_erc20_transfer_calldata(
    token_address: Address,
    provider: &impl Provider,
    to: Address,
    amount: U256,
) -> Result<alloy::rpc::types::TransactionInput, BuildPosTxError> {
    let erc20 = ERC20Token::new(token_address, provider);
    Ok(erc20.transfer(to, amount).calldata().clone().into())
}

fn get_provider(
    chain_id: &Caip2ChainId,
    project_id: &str,
) -> Result<impl Provider, BuildPosTxError> {
    let url = format!(
        "{BASE_URL}?chainId={chain_id}&projectId={project_id}&source={}",
        MessageSource::WalletBuildPosTx,
    )
    .parse()
    .map_err(|_| BuildPosTxError::Validation("Invalid provider URL".to_string()))?;

    Ok(ProviderBuilder::new().on_http(url))
}

pub async fn get_transaction_status(
    _state: State<Arc<AppState>>,
    project_id: &str,
    txid: &str,
    chain_id: &Caip2ChainId,
) -> Result<TransactionStatus, BuildPosTxError> {
    let provider = get_provider(chain_id, project_id)?;

    let txhash = txid
        .parse::<TxHash>()
        .map_err(|e| BuildPosTxError::Validation(format!("Invalid transaction hash: {e}")))?;

    let receipt = provider
        .get_transaction_receipt(txhash)
        .await
        .map_err(|e| {
            BuildPosTxError::Validation(format!("Failed to get transaction receipt: {e}"))
        })?;

    if let Some(receipt) = receipt {
        match receipt.status() {
            true => Ok(TransactionStatus::Confirmed),
            false => Ok(TransactionStatus::Failed),
        }
    } else {
        Ok(TransactionStatus::Pending)
    }
}
