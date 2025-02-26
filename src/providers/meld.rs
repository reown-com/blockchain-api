use {
    super::OnRampMultiProvider,
    crate::{
        error::{RpcError, RpcResult},
        handlers::onramp::{
            multi_quotes::{QueryParams as MultiQuotesQueryParams, QuotesResponse},
            properties::{PropertyType, QueryParams as ProvidersPropertiesQueryParams},
            providers::{ProvidersResponse, QueryParams as ProvidersQueryParams},
            widget::{QueryParams as WidgetQueryParams, SessionData, WidgetResponse},
        },
        providers::ProviderKind,
        Metrics,
    },
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    tracing::log::error,
    url::Url,
};

const BASE_URL: &str = "https://api.meld.io";
const API_VERSION: &str = "2023-12-19";
const DEFAULT_CATEGORY: &str = "CRYPTO_ONRAMP";
const DEFAULT_SESSION_TYPE: &str = "BUY";

#[derive(Debug)]
pub struct MeldProvider {
    pub provider_kind: ProviderKind,
    pub api_key: String,
    pub http_client: reqwest::Client,
}

impl MeldProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            provider_kind: ProviderKind::Meld,
            api_key,
            http_client: reqwest::Client::new(),
        }
    }

    async fn send_get_request(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("Meld-Version", API_VERSION)
            .header("Authorization", format!("BASIC {}", self.api_key))
            .send()
            .await
    }

    async fn send_post_request<T>(
        &self,
        url: Url,
        params: &T,
    ) -> Result<reqwest::Response, reqwest::Error>
    where
        T: Serialize,
    {
        self.http_client
            .post(url)
            .json(&params)
            .header("Meld-Version", API_VERSION)
            .header("Authorization", format!("BASIC {}", self.api_key))
            .send()
            .await
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WidgetRequestParams {
    pub session_data: SessionData,
    pub session_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MeldQuotesResponse {
    pub quotes: Vec<QuotesResponse>,
}

#[async_trait]
impl OnRampMultiProvider for MeldProvider {
    #[tracing::instrument(skip(self), fields(provider = "Meld"), level = "debug")]
    async fn get_providers(
        &self,
        params: ProvidersQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<Vec<ProvidersResponse>> {
        let base = format!("{}/service-providers", BASE_URL);
        let mut url = Url::parse(&base).map_err(|_| RpcError::OnRampParseURLError)?;
        if let Some(countries) = params.countries {
            url.query_pairs_mut().append_pair("countries", &countries);
        }
        url.query_pairs_mut()
            .append_pair("categories", DEFAULT_CATEGORY);

        let latency_start = SystemTime::now();
        let response = self.send_get_request(url).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("onramp_providers".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Meld providers response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::OnRampProviderError);
        }

        Ok(response.json::<Vec<ProvidersResponse>>().await?)
    }

    #[tracing::instrument(skip(self), fields(provider = "Meld"), level = "debug")]
    async fn get_providers_properties(
        &self,
        params: ProvidersPropertiesQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<serde_json::Value> {
        let base_url = match params.r#type {
            PropertyType::Countries => {
                format!("{}/service-providers/properties/countries", BASE_URL)
            }
            PropertyType::CryptoCurrencies => format!(
                "{}/service-providers/properties/crypto-currencies",
                BASE_URL
            ),
            PropertyType::FiatCurrencies => {
                format!("{}/service-providers/properties/fiat-currencies", BASE_URL)
            }
            PropertyType::PaymentMethods => {
                format!("{}/service-providers/properties/payment-methods", BASE_URL)
            }
            PropertyType::FiatPurchasesLimits => format!(
                "{}/service-providers/limits/fiat-currency-purchases",
                BASE_URL
            ),
        };
        let mut url = Url::parse(&base_url).map_err(|_| RpcError::OnRampParseURLError)?;
        if let Some(countries) = params.countries {
            url.query_pairs_mut().append_pair("countries", &countries);
        }
        url.query_pairs_mut()
            .append_pair("categories", DEFAULT_CATEGORY);

        let latency_start = SystemTime::now();
        let response = self.send_get_request(url).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("onramp_providers_properties".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Meld providers properties response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::OnRampProviderError);
        }

        Ok(response.json().await?)
    }

    #[tracing::instrument(skip(self), fields(provider = "Meld"), level = "debug")]
    async fn get_widget(
        &self,
        params: WidgetQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<WidgetResponse> {
        let base = format!("{}/crypto/session/widget", BASE_URL);
        let url = Url::parse(&base).map_err(|_| RpcError::OnRampParseURLError)?;

        let latency_start = SystemTime::now();
        let response = self
            .send_post_request(
                url,
                &WidgetRequestParams {
                    session_type: DEFAULT_SESSION_TYPE.to_string(),
                    session_data: params.session_data,
                },
            )
            .await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("onramp_widget".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Meld get widget url response. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::OnRampProviderError);
        }

        Ok(response.json::<WidgetResponse>().await?)
    }

    async fn get_quotes(
        &self,
        params: MultiQuotesQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<Vec<QuotesResponse>> {
        let base = format!("{}/payments/crypto/quote", BASE_URL);
        let url = Url::parse(&base).map_err(|_| RpcError::OnRampParseURLError)?;

        let latency_start = SystemTime::now();
        let response = self.send_post_request(url, &params).await?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("onramp_multi_quotes".to_string()),
        );

        if !response.status().is_success() {
            error!(
                "Error on Meld get quotes. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::OnRampProviderError);
        }

        let response_quotes = response.json::<MeldQuotesResponse>().await?;

        Ok(response_quotes.quotes)
    }
}
