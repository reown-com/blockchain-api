use {
    super::BalanceProvider,
    crate::{
        error::{RpcError, RpcResult},
        handlers::balance::{BalanceQueryParams, BalanceResponseBody},
        providers::balance::{BalanceItem, BalanceQuantity},
    },
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
    tracing::log::error,
    url::Url,
};

const ACCOUNT_TOKENS_URL: &str = "https://pro-api.solscan.io/v1.0/account/tokens";

#[derive(Debug)]
pub struct SolScanProvider {
    pub api_v1_token: String,
}

impl SolScanProvider {
    pub fn new(api_v1_token: String) -> Self {
        Self { api_v1_token }
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
            .header("Content-Type", "application/json")
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
                chain_id: None,
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
