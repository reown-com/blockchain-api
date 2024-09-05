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
                HistoryTransactionMetadata, HistoryTransactionTransfer,
                HistoryTransactionTransferQuantity,
            },
        },
        utils::crypto::SOLANA_NATIVE_TOKEN_ADDRESS,
    },
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
    std::fmt,
    tracing::log::error,
    url::Url,
};

const SOLANA_SOL_TOKEN_ICON: &str =
    "https://cdn.jsdelivr.net/gh/trustwallet/assets@master/blockchains/solana/info/logo.png";
const SOLANA_MAINNET_CHAIN_ID: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
const ACCOUNT_TOKENS_URL: &str = "https://pro-api.solscan.io/v1.0/account/tokens";
const ACCOUNT_HISTORY_URL: &str = "https://pro-api.solscan.io/v2.0/account/transfer";
const TOKEN_METADATA_URL: &str = "https://pro-api.solscan.io/v2.0/token/meta";

const WSOL_TOKEN_ADDRESS: &str = "So11111111111111111111111111111111111111112";

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
    pub api_v1_token: String,
    pub api_v2_token: String,
    pub http_client: reqwest::Client,
}

impl SolScanProvider {
    pub fn new(api_v1_token: String, api_v2_token: String) -> Self {
        Self {
            api_v1_token,
            api_v2_token,
            http_client: reqwest::Client::new(),
        }
    }

    async fn metadata_token_request(&self, address: &str) -> Result<TokenMetaData, RpcError> {
        let mut url =
            Url::parse(TOKEN_METADATA_URL).map_err(|_| RpcError::FungiblePriceParseURLError)?;
        url.query_pairs_mut().append_pair("address", address);

        let response = self
            .http_client
            .get(url)
            .header("token", self.api_v2_token.clone())
            .send()
            .await?;

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

    async fn get_token_info(&self, address: &str) -> Result<TokenMetaData, RpcError> {
        // Respond instantly for the native token (SOL) metadata with making just a price request
        // since metadata is static
        if address == SOLANA_NATIVE_TOKEN_ADDRESS {
            // Temporary using the WSOL metadata to get the SOL price
            // until the SolScan pricing endpoint is fixed
            let price = self.metadata_token_request(WSOL_TOKEN_ADDRESS).await?.price;

            return Ok(TokenMetaData {
                address: SOLANA_NATIVE_TOKEN_ADDRESS.to_string(),
                name: "Solana".to_string(),
                symbol: "SOL".to_string(),
                decimals: 9,
                icon: Some(SOLANA_SOL_TOKEN_ICON.to_string()),
                price,
            });
        }

        let metadata = self.metadata_token_request(WSOL_TOKEN_ADDRESS).await?;
        Ok(metadata)
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
        http_client: reqwest::Client,
    ) -> RpcResult<BalanceResponseBody> {
        let mut url = Url::parse(ACCOUNT_TOKENS_URL).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut().append_pair("account", &address);

        let response = http_client
            .get(url)
            .header("token", self.api_v1_token.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            error!(
                "Error on SolScan balance response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::BalanceProviderError);
        }
        let body = response.json::<Vec<TokensResponseItem>>().await?;

        let balances_vec = body
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
        http_client: reqwest::Client,
    ) -> RpcResult<HistoryResponseBody> {
        let mut url =
            Url::parse(ACCOUNT_HISTORY_URL).map_err(|_| RpcError::BalanceParseURLError)?;
        url.query_pairs_mut().append_pair("page_size", "100");
        url.query_pairs_mut().append_pair("remove_spam", "true");
        url.query_pairs_mut()
            .append_pair("exclude_amount_zero", "true");
        url.query_pairs_mut().append_pair("address", &address);
        let page = params.cursor.unwrap_or("1".into());
        url.query_pairs_mut().append_pair("page", &page);

        let response = http_client
            .get(url)
            .header("token", self.api_v2_token.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            error!(
                "Error on SolScan transactions history response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::TransactionProviderError);
        }
        let body = response.json::<HistoryResponse>().await?;

        let transactions: Vec<HistoryTransaction> = body
            .data
            .into_iter()
            .map(|f| HistoryTransaction {
                id: f.block_id.to_string(),
                metadata: HistoryTransactionMetadata {
                    operation_type: match f.activity_type {
                        HistoryActivityType::Transfer => {
                            if f.flow == HistoryDirectionType::In {
                                "receive".to_string()
                            } else {
                                "send".to_string()
                            }
                        }
                        HistoryActivityType::Burn => "burn".to_string(),
                        HistoryActivityType::Mint => "mint".to_string(),
                        HistoryActivityType::CreateAccount => "execute".to_string(),
                    },
                    hash: f.trans_id,
                    mined_at: f.time,
                    nonce: 0,
                    sent_from: f.from_address,
                    sent_to: f.to_address,
                    status: "confirmed".to_string(), // Balance changes are always confirmed
                    application: None,
                    chain: Some(SOLANA_MAINNET_CHAIN_ID.to_string()),
                },
                transfers: Some(vec![HistoryTransactionTransfer {
                    fungible_info: None, // Todo: Add fungible info from saved tokens info list
                    nft_info: None,
                    direction: f.flow.to_string(),
                    quantity: HistoryTransactionTransferQuantity {
                        numeric: f.amount.to_string(),
                    },
                    value: None,
                    price: None,
                }]),
            })
            .collect();

        let next = if !transactions.is_empty() {
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
    ) -> RpcResult<PriceResponseBody> {
        if currency != &SupportedCurrencies::USD {
            return Err(RpcError::UnsupportedCurrency(
                "Only USD currency is supported for Solana tokens price".to_string(),
            ));
        }

        let info = self.get_token_info(address).await?;
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
