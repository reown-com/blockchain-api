use {
    crate::{
        error::{RpcError, RpcResult},
        handlers::convert::tokens::{TokenItem, TokensListQueryParams, TokensListResponseBody},
        providers::ConversionProvider,
        utils::crypto,
    },
    async_trait::async_trait,
    serde::Deserialize,
    std::collections::HashMap,
    tracing::log::error,
    url::Url,
};

#[derive(Debug)]
pub struct OneInchProvider {
    pub api_key: String,
    pub base_api_url: String,
    pub http_client: reqwest::Client,
}

impl OneInchProvider {
    pub fn new(api_key: String) -> Self {
        let base_api_url = "https://api.1inch.dev/swap/v6.0".to_string();
        let http_client = reqwest::Client::new();
        Self {
            api_key,
            base_api_url,
            http_client,
        }
    }

    async fn send_request(
        &self,
        url: Url,
        http_client: &reqwest::Client,
    ) -> Result<reqwest::Response, reqwest::Error> {
        http_client
            .get(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
    }
}

#[derive(Debug, Deserialize)]
struct OneInchTokensResponse {
    tokens: HashMap<String, OneInchTokenItem>,
}

#[derive(Debug, Deserialize)]
struct OneInchTokenItem {
    symbol: String,
    name: String,
    address: String,
    decimals: u8,
    #[serde(alias = "logoURI")]
    logo_uri: Option<String>,
    eip2612: Option<bool>,
}

#[async_trait]
impl ConversionProvider for OneInchProvider {
    #[tracing::instrument(skip(self, params), fields(provider = "1inch"))]
    async fn get_tokens_list(
        &self,
        params: TokensListQueryParams,
    ) -> RpcResult<TokensListResponseBody> {
        let evm_chain_id = crypto::disassemble_caip2(&params.chain_id)?.1;
        let base = format!("{}/{}/tokens", &self.base_api_url, evm_chain_id.clone());
        let url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;

        let response = self.send_request(url, &self.http_client.clone()).await?;

        if !response.status().is_success() {
            error!(
                "Error on getting tokens list for conversion from 1inch provider. Status is not \
                 OK: {:?}",
                response.status(),
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<OneInchTokensResponse>().await?;

        let response: TokensListResponseBody = TokensListResponseBody {
            tokens: body
                .tokens
                .into_values()
                .map(|token| TokenItem {
                    name: token.name,
                    symbol: token.symbol,
                    address: crypto::format_to_caip10(
                        crypto::CaipNamespaces::Eip155,
                        &evm_chain_id,
                        &token.address,
                    ),
                    decimals: token.decimals,
                    logo_uri: token.logo_uri,
                    eip2612: if token.eip2612.is_some() {
                        token.eip2612
                    } else {
                        Some(false)
                    },
                })
                .collect(),
        };

        Ok(response)
    }
}
