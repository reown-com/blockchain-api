use {
    super::{
        AssetNamespaceType, BuildPosTxsError, CheckTransactionResult, PaymentIntent,
        SupportedNamespace, TransactionBuilder, TransactionId, TransactionRpc, TransactionStatus,
        ValidatedPaymentIntent,
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
    strum::{EnumIter, IntoEnumIterator},
    strum_macros::{Display, EnumString},
    tracing::debug,
};

const NATIVE_GAS_LIMIT: u64 = 21_000;
const ETH_SEND_TRANSACTION_METHOD: &str = "eth_sendTransaction";
const BASE_URL: &str = "https://rpc.walletconnect.org/v1";
const DEFAULT_CHECK_IN: usize = 1000;
const NAMESPACE_NAME: &str = "eip155";

sol! {
    #[sol(rpc)]
    interface ERC20Token {
        function transfer(address to, uint256 value) external returns (bool);
        function decimals() external view returns (uint8);
    }
}

#[derive(Debug, Clone, PartialEq, EnumString, Display, EnumIter)]
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
    ) -> Result<Self, BuildPosTxsError> {
        let to = recipient
            .parse::<Address>()
            .map_err(|e| BuildPosTxsError::Validation(format!("Invalid recipient: {}", e)))?;

        let from = sender
            .parse::<Address>()
            .map_err(|e| BuildPosTxsError::Validation(format!("Invalid sender: {}", e)))?;

        Ok(Self {
            to,
            from,
            tx_request: TransactionRequest::default(),
            project_id: project_id.to_string(),
            chain_id: chain_id.clone(),
        })
    }

    async fn with_native_transfer(mut self, amount: &str) -> Result<Self, BuildPosTxsError> {
        let wei_value = parse_ether_amount(amount)?;

        self.tx_request = self.tx_request.to(self.to).value(wei_value).from(self.from);

        Ok(self)
    }

    async fn with_erc20_transfer(
        mut self,
        asset_address: &str,
        amount: &str,
    ) -> Result<Self, BuildPosTxsError> {
        let token_address = asset_address
            .parse::<Address>()
            .map_err(|e| BuildPosTxsError::Validation(format!("Invalid asset address: {}", e)))?;
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

    async fn finalize(mut self) -> Result<TransactionRpc, BuildPosTxsError> {
        let provider = get_provider(&self.chain_id, &self.project_id)?;

        let fees = provider
            .estimate_eip1559_fees(None)
            .await
            .map_err(|e| BuildPosTxsError::Validation(format!("Failed to estimate fees: {e}")))?;

        self.tx_request = self
            .tx_request
            .max_fee_per_gas(fees.max_fee_per_gas)
            .max_priority_fee_per_gas(fees.max_priority_fee_per_gas);

        let gas_limit = if has_transaction_data(&self.tx_request) {
            provider
                .estimate_gas(&self.tx_request)
                .await
                .map_err(|e| BuildPosTxsError::Validation(format!("Failed to estimate gas: {e}")))?
        } else {
            NATIVE_GAS_LIMIT
        };

        self.tx_request = self.tx_request.gas_limit(gas_limit);
        debug!("finalized tx: {:?}", self.tx_request);

        Ok(TransactionRpc {
            method: ETH_SEND_TRANSACTION_METHOD.to_string(),
            params: serde_json::json!([self.tx_request]),
            chain_id: self.chain_id.to_string(),
            id: TransactionId::new(&self.chain_id).to_string(),
        })
    }
}

#[async_trait]
impl TransactionBuilder<AssetNamespace> for EvmTransactionBuilder {
    fn namespace(&self) -> &'static str {
        NAMESPACE_NAME
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
        let builder = EvmTxBuilder::new(
            &project_id,
            params.asset.chain_id(),
            &params.recipient_address,
            &params.sender_address,
        )?;

        let tx = match params.namespace {
            AssetNamespace::Slip44 => {
                builder
                    .with_native_transfer(&params.amount)
                    .await?
                    .finalize()
                    .await?
            }
            AssetNamespace::Erc20 => {
                builder
                    .with_erc20_transfer(params.asset.asset_reference(), &params.amount)
                    .await?
                    .finalize()
                    .await?
            }
        };

        Ok(tx)
    }
}

fn parse_ether_amount(amount: &str) -> Result<U256, BuildPosTxsError> {
    let value = parse_units(amount, "ether").map_err(|e| {
        BuildPosTxsError::Validation(format!("Unable to parse amount in ether: {e}"))
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
) -> Result<U256, BuildPosTxsError> {
    let erc20 = ERC20Token::new(token_address, provider);

    let decimals = erc20
        .decimals()
        .call()
        .await
        .map_err(|e| BuildPosTxsError::Validation(format!("Failed to get decimals: {e}")))?
        ._0;

    debug!("decimals: {decimals}");

    let value = parse_units(amount, decimals).map_err(|e| {
        BuildPosTxsError::Validation(format!(
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
) -> Result<alloy::rpc::types::TransactionInput, BuildPosTxsError> {
    let erc20 = ERC20Token::new(token_address, provider);
    Ok(erc20.transfer(to, amount).calldata().clone().into())
}

fn get_provider(
    chain_id: &Caip2ChainId,
    project_id: &str,
) -> Result<impl Provider, BuildPosTxsError> {
    let url = format!(
        "{BASE_URL}?chainId={chain_id}&projectId={project_id}&source={}",
        MessageSource::WalletBuildPosTx,
    )
    .parse()
    .map_err(|_| BuildPosTxsError::Validation("Invalid provider URL".to_string()))?;

    Ok(ProviderBuilder::new().on_http(url))
}

pub async fn get_transaction_status(
    _state: State<Arc<AppState>>,
    project_id: &str,
    txid: &str,
    chain_id: &Caip2ChainId,
) -> Result<TransactionStatus, BuildPosTxsError> {
    let provider = get_provider(chain_id, project_id)?;

    let txhash = txid
        .parse::<TxHash>()
        .map_err(|e| BuildPosTxsError::Validation(format!("Invalid transaction hash: {e}")))?;

    let receipt = provider
        .get_transaction_receipt(txhash)
        .await
        .map_err(|e| {
            BuildPosTxsError::Validation(format!("Failed to get transaction receipt: {e}"))
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

pub async fn check_transaction(
    state: State<Arc<AppState>>,
    project_id: &str,
    txid: &str,
    chain_id: &Caip2ChainId,
) -> Result<CheckTransactionResult, BuildPosTxsError> {
    let status = get_transaction_status(state, project_id, txid, chain_id).await?;

    match status {
        TransactionStatus::Pending => Ok(CheckTransactionResult {
            status,
            check_in: Some(DEFAULT_CHECK_IN),
            txid: Some(txid.to_string()),
        }),
        TransactionStatus::Confirmed => Ok(CheckTransactionResult {
            status,
            check_in: None,
            txid: Some(txid.to_string()),
        }),
        TransactionStatus::Failed => Ok(CheckTransactionResult {
            status,
            check_in: None,
            txid: None,
        }),
    }
}

pub fn get_namespace_info() -> SupportedNamespace {
    SupportedNamespace {
        name: NAMESPACE_NAME.to_string(),
        methods: vec![ETH_SEND_TRANSACTION_METHOD.to_string()],
        events: vec![],
        capabilities: None,
        asset_namespaces: AssetNamespace::iter().map(|x| x.to_string()).collect(),
    }
}
