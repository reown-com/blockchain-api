use {
    super::{
        BalanceProvider, FungiblePriceProvider, HistoryProvider, PriceResponseBody,
        SupportedCurrencies,
    },
    crate::{
        error::{RpcError, RpcResult},
        handlers::{
            balance::{BalanceItem, BalanceQuantity, BalanceQueryParams, BalanceResponseBody},
            fungible_price::FungiblePriceItem,
            history::{
                HistoryQueryParams, HistoryResponseBody, HistoryTransaction,
                HistoryTransactionFungibleInfo, HistoryTransactionMetadata,
                HistoryTransactionTransfer, HistoryTransactionTransferQuantity,
                HistoryTransactionURLItem,
            },
        },
        providers::ProviderKind,
        utils::crypto::SOLANA_NATIVE_TOKEN_ADDRESS,
        Metrics,
    },
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
    std::{fmt, sync::Arc, time::SystemTime},
    tracing::log::error,
    url::Url,
};

const SOLANA_SOL_TOKEN_ICON: &str =
    "https://cdn.jsdelivr.net/gh/trustwallet/assets@master/blockchains/solana/info/logo.png";
const SOLANA_MAINNET_CHAIN_ID: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
const ACCOUNT_TOKENS_URL: &str = "https://pro-api.solscan.io/v1.0/account/tokens";
const ACCOUNT_HISTORY_URL: &str = "https://pro-api.solscan.io/v2.0/account/transfer";
const TOKEN_METADATA_URL: &str = "https://pro-api.solscan.io/v2.0/token/meta";
const ACCOUNT_DETAIL_URL: &str = "https://pro-api.solscan.io/v2.0/account/detail";

const WSOL_TOKEN_ADDRESS: &str = "So11111111111111111111111111111111111111112";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct AccountDetailResponse {
    pub data: AccountDetail,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct AccountDetail {
    pub lamports: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TokenInfoResponse {
    pub data: TokenMetaData,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TokenMetaData {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TokenPriceResponse {
    pub data: Vec<TokenPriceResponseData>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TokenPriceResponseData {
    pub price: f64,
}

#[derive(Debug)]
pub struct SolScanProvider {
    pub provider_kind: ProviderKind,
    pub api_v1_token: String,
    pub api_v2_token: String,
    pub http_client: reqwest::Client,
}

impl SolScanProvider {
    pub fn new(api_v1_token: String, api_v2_token: String) -> Self {
        Self {
            provider_kind: ProviderKind::SolScan,
            api_v1_token,
            api_v2_token,
            http_client: reqwest::Client::new(),
        }
    }

    async fn send_request_v1(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("token", self.api_v1_token.clone())
            .send()
            .await
    }

    async fn send_request_v2(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("token", self.api_v2_token.clone())
            .send()
            .await
    }

    async fn metadata_token_request(
        &self,
        address: &str,
        metrics: Arc<Metrics>,
    ) -> Result<TokenMetaData, RpcError> {
        let mut url =
            Url::parse(TOKEN_METADATA_URL).map_err(|_| RpcError::FungiblePriceParseURLError)?;
        url.query_pairs_mut().append_pair("address", address);

        let latency_start = SystemTime::now();
        let response = self.send_request_v2(url).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some(TOKEN_METADATA_URL.to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on SolScan token metadata response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::FungiblePriceProviderError(
                "Token metadata provider response status is not success".to_string(),
            ));
        }
        let body = response.json::<TokenInfoResponse>().await?;

        Ok(TokenMetaData {
            address: body.data.address,
            name: body.data.name,
            symbol: body.data.symbol,
            decimals: body.data.decimals,
            icon: body.data.icon,
            price: body.data.price,
        })
    }

    async fn get_token_info(
        &self,
        address: &str,
        metrics: Arc<Metrics>,
    ) -> Result<TokenMetaData, RpcError> {
        // Respond instantly for the native token (SOL) metadata with making just a price request
        // since metadata is static
        if address == SOLANA_NATIVE_TOKEN_ADDRESS {
            // Temporary using the WSOL metadata to get the SOL price
            // until the SolScan pricing endpoint is fixed
            let price = self
                .metadata_token_request(WSOL_TOKEN_ADDRESS, metrics)
                .await?
                .price;

            return Ok(TokenMetaData {
                address: SOLANA_NATIVE_TOKEN_ADDRESS.to_string(),
                name: "Solana".to_string(),
                symbol: "SOL".to_string(),
                decimals: 9,
                icon: Some(SOLANA_SOL_TOKEN_ICON.to_string()),
                price,
            });
        }

        let metadata = self
            .metadata_token_request(WSOL_TOKEN_ADDRESS, metrics)
            .await?;
        Ok(metadata)
    }

    // Get SOL address balance by getting account detail
    async fn get_sol_balance(&self, address: &str, metrics: Arc<Metrics>) -> Result<f64, RpcError> {
        let mut url = Url::parse(ACCOUNT_DETAIL_URL).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut().append_pair("address", address);

        let latency_start = SystemTime::now();
        let response = self.send_request_v2(url).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some(ACCOUNT_DETAIL_URL.to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on SolScan account detail response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::BalanceProviderError);
        }
        let detail = response.json::<AccountDetailResponse>().await?;

        let lamports = detail.data.lamports.unwrap_or_default();
        let balance = lamports as f64 / 10f64.powf(9.0);

        Ok(balance)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
struct TokensResponseItem {
    pub token_address: String,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_icon: Option<String>,
    pub token_amount: AmountItem,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
struct AmountItem {
    pub amount: String,
    pub decimals: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct HistoryResponse {
    pub success: bool,
    pub data: Vec<HistoryResponseItem>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct HistoryResponseItem {
    pub block_time: usize,
    pub block_id: usize,
    pub trans_id: String,
    pub activity_type: HistoryActivityType,
    pub from_address: String,
    pub to_address: String,
    pub token_address: String,
    pub token_decimals: u8,
    pub amount: usize,
    pub flow: HistoryDirectionType,
    pub time: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
enum HistoryDirectionType {
    In,
    Out,
}
impl fmt::Display for HistoryDirectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HistoryDirectionType::In => write!(f, "in"),
            HistoryDirectionType::Out => write!(f, "out"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum HistoryActivityType {
    #[serde(rename = "ACTIVITY_SPL_TRANSFER")]
    Transfer,
    #[serde(rename = "ACTIVITY_SPL_BURN")]
    Burn,
    #[serde(rename = "ACTIVITY_SPL_MINT")]
    Mint,
    #[serde(rename = "ACTIVITY_SPL_CREATE_ACCOUNT")]
    CreateAccount,
}

#[async_trait]
impl BalanceProvider for SolScanProvider {
    #[tracing::instrument(skip(self), fields(provider = "SolScan"), level = "debug")]
    async fn get_balance(
        &self,
        address: String,
        _params: BalanceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<BalanceResponseBody> {
        let mut url = Url::parse(ACCOUNT_TOKENS_URL).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut().append_pair("account", &address);

        let latency_start = SystemTime::now();
        let response = self.send_request_v1(url).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some(ACCOUNT_TOKENS_URL.to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on SolScan balance response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::BalanceProviderError);
        }
        let body = response.json::<Vec<TokensResponseItem>>().await?;

        let mut balances_vec: Vec<BalanceItem> = body
            .into_iter()
            .map(|f| BalanceItem {
                name: f
                    .token_name
                    .unwrap_or_else(|| f.token_symbol.clone().unwrap_or_default()),
                symbol: f.token_symbol.unwrap_or_default(),
                chain_id: Some(SOLANA_MAINNET_CHAIN_ID.to_string()),
                address: Some(f.token_address),
                value: None,
                price: 0.0,
                quantity: BalanceQuantity {
                    decimals: f.token_amount.decimals.to_string(),
                    numeric: f.token_amount.amount,
                },
                icon_url: f.token_icon.unwrap_or_default(),
            })
            .collect();

        // Inject Solana native token (SOL) balance if not zero
        let sol_balance = self.get_sol_balance(&address, metrics.clone()).await?;
        if sol_balance > 0.0 {
            let sol_metadata = self
                .get_token_info(SOLANA_NATIVE_TOKEN_ADDRESS, metrics)
                .await?;
            let sol_balance_item = BalanceItem {
                name: sol_metadata.name,
                symbol: sol_metadata.symbol,
                chain_id: Some(SOLANA_MAINNET_CHAIN_ID.to_string()),
                address: sol_metadata.address.into(),
                value: Some(sol_balance * sol_metadata.price),
                price: sol_metadata.price,
                quantity: BalanceQuantity {
                    decimals: sol_metadata.decimals.to_string(),
                    numeric: sol_balance.to_string(),
                },
                icon_url: sol_metadata.icon.unwrap_or_default(),
            };
            balances_vec.push(sol_balance_item);
        }

        let response = BalanceResponseBody {
            balances: balances_vec,
        };

        Ok(response)
    }
}

#[async_trait]
impl HistoryProvider for SolScanProvider {
    #[tracing::instrument(skip(self, params), fields(provider = "SolScan"), level = "debug")]
    async fn get_transactions(
        &self,
        address: String,
        params: HistoryQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<HistoryResponseBody> {
        let page_size = 100;
        let mut url =
            Url::parse(ACCOUNT_HISTORY_URL).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut()
            .append_pair("page_size", &page_size.to_string());
        url.query_pairs_mut().append_pair("remove_spam", "true");
        url.query_pairs_mut()
            .append_pair("exclude_amount_zero", "true");
        url.query_pairs_mut().append_pair("address", &address);
        let page = params.cursor.unwrap_or("1".into());
        url.query_pairs_mut().append_pair("page", &page);

        let latency_start = SystemTime::now();
        let response = self.send_request_v2(url).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some(ACCOUNT_HISTORY_URL.to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on SolScan transactions history response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::TransactionProviderError);
        }
        let body = response.json::<HistoryResponse>().await?;

        let mut transactions: Vec<HistoryTransaction> = Vec::new();
        for item in &body.data {
            let token_info = self
                .get_token_info(&item.token_address, metrics.clone())
                .await?;
            let decimal_amount = item.amount as f64 / 10f64.powf(token_info.decimals as f64);
            let transaction = HistoryTransaction {
                id: item.block_id.to_string(),
                metadata: HistoryTransactionMetadata {
                    operation_type: match item.activity_type {
                        HistoryActivityType::Transfer => {
                            if item.flow == HistoryDirectionType::In {
                                "receive".to_string()
                            } else {
                                "send".to_string()
                            }
                        }
                        HistoryActivityType::Burn => "burn".to_string(),
                        HistoryActivityType::Mint => "mint".to_string(),
                        HistoryActivityType::CreateAccount => "execute".to_string(),
                    },
                    hash: item.trans_id.clone(),
                    mined_at: item.time.clone(),
                    nonce: 0,
                    sent_from: item.from_address.clone(),
                    sent_to: item.to_address.clone(),
                    status: "confirmed".to_string(), // Balance changes are always confirmed
                    application: None,
                    chain: Some(SOLANA_MAINNET_CHAIN_ID.to_string()),
                },
                transfers: Some(vec![HistoryTransactionTransfer {
                    fungible_info: Some(HistoryTransactionFungibleInfo {
                        name: Some(token_info.name),
                        symbol: Some(token_info.symbol),
                        icon: Some(HistoryTransactionURLItem {
                            url: token_info.icon.unwrap_or_default(),
                        }),
                    }),
                    nft_info: None,
                    direction: item.flow.to_string(),
                    quantity: HistoryTransactionTransferQuantity {
                        numeric: decimal_amount.to_string(),
                    },
                    value: Some(decimal_amount * token_info.price),
                    price: Some(token_info.price),
                }]),
            };
            transactions.push(transaction);
        }

        let next = if !transactions.is_empty() && body.data.len() == page_size {
            Some((page.parse::<u64>().unwrap_or(1) + 1).to_string())
        } else {
            None
        };

        Ok(HistoryResponseBody {
            data: transactions,
            next,
        })
    }
}

#[async_trait]
impl FungiblePriceProvider for SolScanProvider {
    #[tracing::instrument(skip(self), fields(provider = "SolScan"), level = "debug")]
    async fn get_price(
        &self,
        chain_id: &str,
        address: &str,
        currency: &SupportedCurrencies,
        metrics: Arc<Metrics>,
    ) -> RpcResult<PriceResponseBody> {
        if currency != &SupportedCurrencies::USD {
            return Err(RpcError::UnsupportedCurrency(
                "Only USD currency is supported for Solana tokens price".to_string(),
            ));
        }

        let info = self.get_token_info(address, metrics).await?;
        let response = PriceResponseBody {
            fungibles: vec![FungiblePriceItem {
                name: info.name,
                symbol: info.symbol,
                icon_url: info.icon.unwrap_or_default(),
                price: info.price,
                decimals: info.decimals as u32,
            }],
        };

        Ok(response)
    }
}
