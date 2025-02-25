use {
    super::OnRampMultiProvider,
    crate::{
        error::{RpcError, RpcResult},
        handlers::onramp::providers::{ProvidersResponse, QueryParams as ProvidersQueryParams},
        providers::ProviderKind,
        Metrics,
    },
    async_trait::async_trait,
    std::{sync::Arc, time::SystemTime},
    tracing::log::error,
    url::Url,
};

const BASE_URL: &str = "https://api-sb.meld.io";
const API_VERSION: &str = "2023-12-19";
const DEFAULT_CATEGORY: &str = "CRYPTO_ONRAMP";

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
}

#[async_trait]
impl OnRampMultiProvider for MeldProvider {
    #[tracing::instrument(skip(self), fields(provider = "Meld"), level = "debug")]
    async fn get_providers(
        &self,
        params: ProvidersQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<ProvidersResponse> {
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

        Ok(response.json::<ProvidersResponse>().await?)
    }
}
