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
    reqwest::StatusCode,
    serde::{Deserialize, Serialize},
    std::{sync::Arc, time::SystemTime},
    tracing::log::error,
    url::Url,
};

const API_VERSION: &str = "2023-12-19";
const DEFAULT_CATEGORY: &str = "CRYPTO_ONRAMP";
const DEFAULT_SESSION_TYPE: &str = "BUY";

#[derive(Debug)]
pub struct MeldProvider {
    pub provider_kind: ProviderKind,
    pub api_key: String,
    pub api_base_url: String,
    pub http_client: reqwest::Client,
}

impl MeldProvider {
    pub fn new(api_base_url: String, api_key: String) -> Self {
        Self {
            provider_kind: ProviderKind::Meld,
            api_key,
            api_base_url,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MeldErrorResponse {
    pub code: String,
    pub message: String,
}

#[async_trait]
impl OnRampMultiProvider for MeldProvider {
    #[tracing::instrument(skip(self), fields(provider = "Meld"), level = "debug")]
    async fn get_providers(
        &self,
        params: ProvidersQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<Vec<ProvidersResponse>> {
        let base = format!("{}/service-providers", self.api_base_url);
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
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if matches!(
                response.status(),
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ) {
                let response_error = match response.json::<MeldErrorResponse>().await {
                    Ok(response_error) => response_error.message,
                    Err(e) => {
                        error!(
                            "Error parsing Meld HTTP 400 Bad Request error response {:?}",
                            e
                        );
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }
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
                format!(
                    "{}/service-providers/properties/countries",
                    self.api_base_url
                )
            }
            PropertyType::CryptoCurrencies => format!(
                "{}/service-providers/properties/crypto-currencies",
                self.api_base_url
            ),
            PropertyType::FiatCurrencies => {
                format!(
                    "{}/service-providers/properties/fiat-currencies",
                    self.api_base_url
                )
            }
            PropertyType::PaymentMethods => {
                format!(
                    "{}/service-providers/properties/payment-methods",
                    self.api_base_url
                )
            }
            PropertyType::FiatPurchasesLimits => format!(
                "{}/service-providers/limits/fiat-currency-purchases",
                self.api_base_url
            ),
        };
        let mut url = Url::parse(&base_url).map_err(|_| RpcError::OnRampParseURLError)?;
        if let Some(countries) = params.countries {
            url.query_pairs_mut().append_pair("countries", &countries);
        }
        url.query_pairs_mut()
            .append_pair("categories", DEFAULT_CATEGORY);

        let latency_start = SystemTime::now();
        let response = self.send_get_request(url).await.map_err(|e| {
            error!(
                "Error sending request to Meld providers properties: {:?}",
                e
            );
            RpcError::OnRampProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("onramp_providers_properties".to_string()),
        );

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if matches!(
                response.status(),
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ) {
                let response_error = match response.json::<MeldErrorResponse>().await {
                    Ok(response_error) => response_error.message,
                    Err(e) => {
                        error!(
                            "Error parsing Meld HTTP 400 Bad Request error response {:?}",
                            e
                        );
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }
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
        let base = format!("{}/crypto/session/widget", self.api_base_url);
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
            .await
            .map_err(|e| {
                error!("Error sending request to Meld get widget: {:?}", e);
                RpcError::OnRampProviderError
            })?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("onramp_widget".to_string()),
        );

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if matches!(
                response.status(),
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ) {
                let response_error = match response.json::<MeldErrorResponse>().await {
                    Ok(response_error) => response_error.message,
                    Err(e) => {
                        error!(
                            "Error parsing Meld HTTP 400 Bad Request error response {:?}",
                            e
                        );
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }
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
        let base = format!("{}/payments/crypto/quote", self.api_base_url);
        let url = Url::parse(&base).map_err(|_| RpcError::OnRampParseURLError)?;

        let latency_start = SystemTime::now();
        let response = self.send_post_request(url, &params).await.map_err(|e| {
            error!("Error sending request to Meld get quotes: {:?}", e);
            RpcError::OnRampProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("onramp_multi_quotes".to_string()),
        );

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if matches!(
                response.status(),
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ) {
                let response_error = match response.json::<MeldErrorResponse>().await {
                    Ok(response_error) => response_error,
                    Err(e) => {
                        error!(
                            "Error parsing Meld HTTP 400 Bad Request error response {:?}",
                            e
                        );
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        MeldErrorResponse {
                            code: "BAD_REQUEST".to_string(),
                            message: "Invalid parameter".to_string(),
                        }
                    }
                };
                return Err(RpcError::ConversionInvalidParameterWithCode(
                    response_error.code,
                    response_error.message,
                ));
            }

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
