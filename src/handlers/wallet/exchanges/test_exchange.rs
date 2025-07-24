use {
    crate::handlers::wallet::exchanges::{
        BuyTransactionStatus, ExchangeError, ExchangeProvider, GetBuyStatusParams,
        GetBuyStatusResponse, GetBuyUrlParams,
    },
    crate::state::AppState,
    crate::utils::crypto::Caip19Asset,
    axum::extract::State,
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
};

pub struct TestExchange;

const TEST_EXCHANGE_URL: &str = "https://appkit-pay-test-exchange.reown.com";

static CAIP_19_SUPPORTED_ASSETS: Lazy<Vec<Caip19Asset>> = Lazy::new(|| {
    vec![
        Caip19Asset::parse("eip155:11155111/slip44:60").unwrap(),
        Caip19Asset::parse("eip155:84532/slip44:60").unwrap(),
    ]
});

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestExchangeApiResponse {
    pub status: String,
    pub txid: Option<String>,
    pub created_at: Option<String>,
}

impl ExchangeProvider for TestExchange {
    fn id(&self) -> &'static str {
        "reown_test"
    }

    fn name(&self) -> &'static str {
        "Reown Test Exchange"
    }

    fn image_url(&self) -> Option<&'static str> {
        Some("https://pay-assets.reown.com/reown_test_128_128.webp")
    }

    fn is_asset_supported(&self, asset: &Caip19Asset) -> bool {
        CAIP_19_SUPPORTED_ASSETS.contains(asset)
    }
}

impl TestExchange {
    pub fn get_buy_url(
        &self,
        _state: State<Arc<AppState>>,
        params: GetBuyUrlParams,
    ) -> Result<String, ExchangeError> {
        Ok(format!(
            "{}/?asset={}&amount={}&recipient={}&sessionId={}&projectId={}",
            TEST_EXCHANGE_URL,
            params.asset,
            params.amount,
            params.recipient,
            params.session_id,
            params.project_id
        ))
    }

    pub async fn get_buy_status(
        &self,
        state: State<Arc<AppState>>,
        params: GetBuyStatusParams,
    ) -> Result<GetBuyStatusResponse, ExchangeError> {
        let response = state
            .http_client
            .get(format!(
                "{}/api/status?sessionId={}",
                TEST_EXCHANGE_URL, params.session_id
            ))
            .send()
            .await
            .map_err(|e| ExchangeError::GetPayUrlError(e.to_string()))?;

        if response.status().is_success() {
            let api_response: TestExchangeApiResponse = response.json().await.map_err(|e| {
                ExchangeError::GetPayUrlError(format!("Failed to parse response: {}", e))
            })?;

            let status = match api_response.status.to_lowercase().as_str() {
                "success" => BuyTransactionStatus::Success,
                "pending" | "in_progress" => BuyTransactionStatus::InProgress,
                "failed" | "error" => BuyTransactionStatus::Failed,
                _ => BuyTransactionStatus::Unknown,
            };

            Ok(GetBuyStatusResponse {
                status,
                tx_hash: api_response.txid,
            })
        } else {
            Err(ExchangeError::GetPayUrlError(format!(
                "API returned error status: {}",
                response.status()
            )))
        }
    }
}
