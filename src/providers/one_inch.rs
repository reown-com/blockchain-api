use {
    crate::{
        error::{RpcError, RpcResult},
        handlers::{
            convert::{
                allowance::{AllowanceQueryParams, AllowanceResponseBody},
                approve::{
                    ConvertApproveQueryParams, ConvertApproveResponseBody, ConvertApproveTx,
                    ConvertApproveTxEip155,
                },
                gas_price::{GasPriceQueryParams, GasPriceQueryResponseBody},
                quotes::{ConvertQuoteQueryParams, ConvertQuoteResponseBody, QuoteItem},
                tokens::{TokenItem, TokensListQueryParams, TokensListResponseBody},
                transaction::{
                    ConvertTransactionQueryParams, ConvertTransactionResponseBody, ConvertTx,
                    ConvertTxEip155,
                },
            },
            fungible_price::FungiblePriceItem,
            SupportedCurrencies,
        },
        providers::{
            ConversionProvider, FungiblePriceProvider, PriceResponseBody, ProviderKind,
            TokenMetadataCacheProvider,
        },
        utils::crypto,
        Metrics,
    },
    async_trait::async_trait,
    serde::Deserialize,
    std::{collections::HashMap, sync::Arc, time::SystemTime},
    tracing::log::error,
    url::Url,
};

const ONEINCH_FEE: f64 = 0.85;
const GAS_ESTIMATION_SLIPPAGE: f64 = 1.1; // Increase the estimated gas by 10%

#[derive(Debug)]
pub struct OneInchProvider {
    pub provider_kind: ProviderKind,
    pub api_key: String,
    pub referrer: Option<String>,
    pub base_api_url: String,
    pub http_client: reqwest::Client,
}

impl OneInchProvider {
    pub fn new(api_key: String, referrer: Option<String>) -> Self {
        let base_api_url = "https://api.1inch.dev".to_string();
        let http_client = reqwest::Client::new();
        Self {
            provider_kind: ProviderKind::OneInch,
            api_key,
            referrer,
            base_api_url,
            http_client,
        }
    }

    async fn send_request(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
    }

    pub async fn get_token_price(
        &self,
        chain_id: &str,
        address: &str,
        currency: &SupportedCurrencies,
        metrics: Arc<Metrics>,
    ) -> Result<String, RpcError> {
        let address = address.to_lowercase();
        let mut url = Url::parse(
            format!("{}/price/v1.1/{}/{}", &self.base_api_url, chain_id, address).as_str(),
        )
        .map_err(|_| RpcError::ConversionParseURLError)?;
        url.query_pairs_mut()
            .append_pair("currency", &currency.to_string());

        let latency_start = SystemTime::now();
        let price_response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to 1inch provider for fungible price: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            price_response.status().into(),
            latency_start,
            Some(chain_id.to_string()),
            Some("price".to_string()),
        );

        if !price_response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if price_response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = match price_response.json::<OneInchErrorResponse>().await {
                    Ok(response_error) => response_error.description,
                    Err(e) => {
                        error!("Error parsing OneInch HTTP 400 Bad Request error response {e:?}");
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }
            // 404 response is expected when the asset is not supported
            if price_response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(RpcError::AssetNotSupported(address.clone()));
            }

            error!(
                "Error on getting fungible price from 1inch provider. Status is not OK: {:?}",
                price_response.status(),
            );
            return Err(RpcError::ConversionProviderError);
        }
        let price_body = price_response.json::<HashMap<String, String>>().await?;
        price_body.get(&address).map_or(
            {
                error!(
                    "Error on getting fungible price from 1inch provider. Price not found for the \
                     address"
                );
                Err(RpcError::ConversionProviderError)
            },
            |price| Ok(price.clone()),
        )
    }

    async fn get_token_info(
        &self,
        chain_id: &str,
        address: &str,
        metrics: Arc<Metrics>,
    ) -> Result<OneInchTokenItem, RpcError> {
        let address = address.to_lowercase();
        let url = Url::parse(
            format!(
                "{}/token/v1.2/{}/custom/{}",
                &self.base_api_url, chain_id, address
            )
            .as_str(),
        )
        .map_err(|_| RpcError::ConversionParseURLError)?;

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to 1inch provider for token info: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            Some(chain_id.to_string()),
            Some("custom_token_info".to_string()),
        );

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = match response.json::<OneInchErrorResponse>().await {
                    Ok(response_error) => response_error.description,
                    Err(e) => {
                        error!("Error parsing OneInch HTTP 400 Bad Request error response {e:?}");
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }
            // 404 response is expected when the asset is not supported
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(RpcError::AssetNotSupported(address.clone()));
            }

            error!(
                "Error on getting token info from 1inch provider. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<OneInchTokenItem>().await?;
        Ok(body)
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchQuoteResponse {
    dst_amount: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchApproveTxResponse {
    data: String,
    gas_price: String,
    to: String,
    value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchTxResponse {
    dst_amount: String,
    tx: OneInchTxTransaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchTxTransaction {
    from: String,
    to: String,
    data: String,
    gas: usize,
    gas_price: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchErrorResponse {
    description: String,
}
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OneInchGasPriceResponse {
    NonEip1559(OneInchGasPrice),
    Eip1559(OneInchGasPriceEip1559),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchGasPriceEip1559 {
    medium: OneInchGasPriceEip1559Item,
    high: OneInchGasPriceEip1559Item,
    instant: OneInchGasPriceEip1559Item,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchGasPriceEip1559Item {
    max_fee_per_gas: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchGasPrice {
    standard: String,
    fast: String,
    instant: String,
}

#[derive(Debug, Deserialize)]
struct OneInchAllowanceResponse {
    allowance: String,
}

#[async_trait]
impl ConversionProvider for OneInchProvider {
    #[tracing::instrument(skip(self), fields(provider = "1inch"), level = "debug")]
    async fn get_tokens_list(
        &self,
        params: TokensListQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<TokensListResponseBody> {
        let evm_chain_id = crypto::disassemble_caip2(&params.chain_id)?.1;
        let base = format!(
            "{}/swap/v6.0/{}/tokens",
            &self.base_api_url,
            evm_chain_id.clone()
        );
        let url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to 1inch provider for token list: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            Some(evm_chain_id.to_string()),
            Some("tokens_list".to_string()),
        );

        if !response.status().is_success() {
            // 404 response is expected when the chain ID is not supported
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(RpcError::ConversionInvalidParameter(format!(
                    "Chain ID {} is not supported",
                    params.chain_id
                )));
            };

            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = match response.json::<OneInchErrorResponse>().await {
                    Ok(response_error) => response_error.description,
                    Err(e) => {
                        error!("Error parsing OneInch HTTP 400 Bad Request error response {e:?}");
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }

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
                .filter(|token| {
                    if let Some(address) = &params.address {
                        token.address == *address
                    } else {
                        true
                    }
                })
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

    #[tracing::instrument(skip(self), fields(provider = "1inch"), level = "debug")]
    async fn get_convert_quote(
        &self,
        params: ConvertQuoteQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<ConvertQuoteResponseBody> {
        let (_, chain_id, src_address) = crypto::disassemble_caip10(&params.from)?;
        let (_, dst_chain_id, dst_address) = crypto::disassemble_caip10(&params.to)?;

        // Check if from and to chain ids are different
        // 1inch provider does not support cross-chain swaps
        if dst_chain_id != chain_id {
            return Err(RpcError::InvalidParameter(
                "`from` and `to` chain IDs must have the same value".into(),
            ));
        }

        let base = format!(
            "{}/swap/v6.0/{}/quote",
            &self.base_api_url,
            chain_id.clone()
        );
        let mut url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;

        url.query_pairs_mut()
            .append_pair("src", &src_address.to_lowercase());
        url.query_pairs_mut()
            .append_pair("dst", &dst_address.to_lowercase());
        url.query_pairs_mut().append_pair("amount", &params.amount);
        if let Some(referrer) = &self.referrer {
            url.query_pairs_mut().append_pair("referrer", referrer);
            url.query_pairs_mut()
                .append_pair("fee", ONEINCH_FEE.to_string().as_str());
        }
        if let Some(gas_price) = &params.gas_price {
            url.query_pairs_mut().append_pair("gasPrice", gas_price);
        }

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to 1inch provider for convertion quote: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            Some(chain_id.to_string()),
            Some("quote".to_string()),
        );

        let response_status = response.status();
        if !response_status.is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = match response.json::<OneInchErrorResponse>().await {
                    Ok(response_error) => response_error.description,
                    Err(e) => {
                        error!("Error parsing OneInch HTTP 400 Bad Request error response {e:?}");
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }

            error!(
                "Error on getting quotes for conversion from 1inch provider. Status is not OK: \
                 {response_status:?}",
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<OneInchQuoteResponse>().await?;

        let response = ConvertQuoteResponseBody {
            quotes: vec![QuoteItem {
                id: None,
                from_amount: params.amount,
                from_account: params.from,
                to_amount: body.dst_amount,
                to_account: params.to,
            }],
        };

        Ok(response)
    }

    #[tracing::instrument(skip(self), fields(provider = "1inch"), level = "debug")]
    async fn build_approve_tx(
        &self,
        params: ConvertApproveQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<ConvertApproveResponseBody> {
        let chain_id = crypto::disassemble_caip10(&params.from)?.1;
        let (_, dst_chain_id, dst_address) = crypto::disassemble_caip10(&params.to)?;

        // Check if from and to chain ids are different
        // 1inch provider does not support cross-chain swaps
        if dst_chain_id != chain_id {
            return Err(RpcError::InvalidParameter(
                "`from` and `to` chain IDs must have the same value".into(),
            ));
        }

        let base = format!(
            "{}/swap/v6.0/{}/approve/transaction",
            &self.base_api_url, chain_id
        );
        let mut url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;

        url.query_pairs_mut()
            .append_pair("tokenAddress", &dst_address.to_lowercase());
        if let Some(amount) = &params.amount {
            url.query_pairs_mut().append_pair("amount", amount);
        }

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to 1inch provider for building approval tx: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            Some(chain_id.to_string()),
            Some("approve_transactions".to_string()),
        );

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = match response.json::<OneInchErrorResponse>().await {
                    Ok(response_error) => response_error.description,
                    Err(e) => {
                        error!("Error parsing OneInch HTTP 400 Bad Request error response {e:?}");
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }

            error!(
                "Error on building approval tx for conversion from 1inch provider. Status is not \
                 OK: {:?}",
                response.status(),
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<OneInchApproveTxResponse>().await?;

        let response = ConvertApproveResponseBody {
            tx: ConvertApproveTx {
                from: params.from,
                to: crypto::format_to_caip10(crypto::CaipNamespaces::Eip155, &chain_id, &body.to),
                data: body.data,
                value: body.value,
                eip155: Some(ConvertApproveTxEip155 {
                    gas_price: body.gas_price,
                }),
            },
        };

        Ok(response)
    }

    #[tracing::instrument(skip(self), fields(provider = "1inch"), level = "debug")]
    async fn build_convert_tx(
        &self,
        params: ConvertTransactionQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<ConvertTransactionResponseBody> {
        let (_, chain_id, src_address) = crypto::disassemble_caip10(&params.from)?;
        let (_, dst_chain_id, dst_address) = crypto::disassemble_caip10(&params.to)?;
        let (_, user_chain_id, user_address) = crypto::disassemble_caip10(&params.user_address)?;

        // Check if from, to and user chain ids are different
        // 1inch provider does not support cross-chain swaps
        if (dst_chain_id != chain_id) || (user_chain_id != chain_id) {
            return Err(RpcError::InvalidParameter(
                "`from`, `to` and `userAddress` chain IDs must have the same value".into(),
            ));
        }

        let base = format!("{}/swap/v6.0/{}/swap", &self.base_api_url, chain_id);
        let mut url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;

        url.query_pairs_mut()
            .append_pair("src", &src_address.to_lowercase());
        url.query_pairs_mut()
            .append_pair("dst", &dst_address.to_lowercase());
        url.query_pairs_mut().append_pair("amount", &params.amount);
        url.query_pairs_mut()
            .append_pair("from", &user_address.to_lowercase());
        if let Some(referrer) = &self.referrer {
            url.query_pairs_mut().append_pair("referrer", referrer);
            url.query_pairs_mut()
                .append_pair("fee", ONEINCH_FEE.to_string().as_str());
        }
        if let Some(disable_estimate) = &params.disable_estimate {
            url.query_pairs_mut()
                .append_pair("disableEstimate", &disable_estimate.to_string());
        }

        if let Some(eip155) = &params.eip155 {
            url.query_pairs_mut()
                .append_pair("slippage", &eip155.slippage.to_string());
            if let Some(permit) = &eip155.permit {
                url.query_pairs_mut().append_pair("permit", permit);
            }
        } else {
            return Err(RpcError::InvalidParameter(
                "slippage parameter is necessary for this type of conversion".into(),
            ));
        }

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to 1inch provider for building convertion tx: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            Some(chain_id.to_string()),
            Some("swap".to_string()),
        );

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = match response.json::<OneInchErrorResponse>().await {
                    Ok(response_error) => response_error.description,
                    Err(e) => {
                        error!("Error parsing OneInch HTTP 400 Bad Request error response {e:?}");
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }

            error!(
                "Error on building convert tx from 1inch provider. Status is not OK: {:?}",
                response.status(),
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<OneInchTxResponse>().await?;

        let response = ConvertTransactionResponseBody {
            tx: ConvertTx {
                from: crypto::format_to_caip10(
                    crypto::CaipNamespaces::Eip155,
                    &chain_id,
                    &body.tx.from,
                ),
                to: crypto::format_to_caip10(
                    crypto::CaipNamespaces::Eip155,
                    &chain_id,
                    &body.tx.to,
                ),
                data: body.tx.data,
                amount: body.dst_amount,
                eip155: Some(ConvertTxEip155 {
                    gas: (f64::ceil(body.tx.gas as f64 * GAS_ESTIMATION_SLIPPAGE) as usize)
                        .to_string(),
                    gas_price: body.tx.gas_price,
                }),
            },
        };

        Ok(response)
    }

    #[tracing::instrument(skip(self, params), fields(provider = "1inch"), level = "debug")]
    async fn get_gas_price(
        &self,
        params: GasPriceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<GasPriceQueryResponseBody> {
        let evm_chain_id = crypto::disassemble_caip2(&params.chain_id)?.1;
        let base = format!(
            "{}/gas-price/v1.5/{}",
            &self.base_api_url,
            evm_chain_id.clone()
        );
        let url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to 1inch provider for gas price: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            Some(evm_chain_id.to_string()),
            Some("gas_price".to_string()),
        );

        if !response.status().is_success() {
            // 404 response is expected when the chain ID is not supported
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(RpcError::ConversionInvalidParameter(format!(
                    "Chain ID {} is not supported",
                    params.chain_id
                )));
            };

            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = match response.json::<OneInchErrorResponse>().await {
                    Ok(response_error) => response_error.description,
                    Err(e) => {
                        error!("Error parsing OneInch HTTP 400 Bad Request error response {e:?}");
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }

            error!(
                "Error on getting gas price for conversion from 1inch provider. Status is not OK: \
                 {:?}",
                response.status(),
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<OneInchGasPriceResponse>().await?;

        match body {
            OneInchGasPriceResponse::NonEip1559(gas_price) => Ok(GasPriceQueryResponseBody {
                standard: gas_price.standard,
                fast: gas_price.fast,
                instant: gas_price.instant,
            }),
            OneInchGasPriceResponse::Eip1559(gas_price) => Ok(GasPriceQueryResponseBody {
                standard: gas_price.medium.max_fee_per_gas,
                fast: gas_price.high.max_fee_per_gas,
                instant: gas_price.instant.max_fee_per_gas,
            }),
        }
    }

    #[tracing::instrument(skip(self), fields(provider = "1inch"), level = "debug")]
    async fn get_allowance(
        &self,
        params: AllowanceQueryParams,
        metrics: Arc<Metrics>,
    ) -> RpcResult<AllowanceResponseBody> {
        let (_, evm_chain_id, token_address) = crypto::disassemble_caip10(&params.token_address)?;
        let wallet_address = crypto::disassemble_caip10(&params.user_address)?.2;
        let base = format!(
            "{}/swap/v6.0/{}/approve/allowance",
            &self.base_api_url,
            evm_chain_id.clone()
        );
        let mut url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;
        url.query_pairs_mut()
            .append_pair("tokenAddress", &token_address.to_lowercase());
        url.query_pairs_mut()
            .append_pair("walletAddress", &wallet_address.to_lowercase());

        let latency_start = SystemTime::now();
        let response = self.send_request(url).await.map_err(|e| {
            error!("Error sending request to 1inch provider for allowance: {e:?}");
            RpcError::ConversionProviderError
        })?;
        metrics.add_latency_and_status_code_for_provider(
            &self.provider_kind,
            response.status().into(),
            latency_start,
            Some(evm_chain_id.to_string()),
            Some("allowance".to_string()),
        );

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = match response.json::<OneInchErrorResponse>().await {
                    Ok(response_error) => response_error.description,
                    Err(e) => {
                        error!("Error parsing OneInch HTTP 400 Bad Request error response {e:?}");
                        // Respond to the client with a generic error message and HTTP 400 anyway
                        "Invalid parameter".to_string()
                    }
                };
                return Err(RpcError::ConversionInvalidParameter(response_error));
            }
            // 404 response is expected when the token address is not supported
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(RpcError::ConversionInvalidParameter(format!(
                    "token {} is not supported",
                    params.token_address
                )));
            };

            error!(
                "Error on getting allowance for conversion from 1inch provider. Status is not OK: \
                 {:?}",
                response.status(),
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<OneInchAllowanceResponse>().await?;
        Ok(AllowanceResponseBody {
            allowance: body.allowance,
        })
    }
}

#[async_trait]
impl FungiblePriceProvider for OneInchProvider {
    async fn get_price(
        &self,
        chain_id: &str,
        address: &str,
        currency: &SupportedCurrencies,
        _metadata_cache: &Arc<dyn TokenMetadataCacheProvider>,
        metrics: Arc<Metrics>,
    ) -> RpcResult<PriceResponseBody> {
        let price = self
            .get_token_price(chain_id, address, currency, metrics.clone())
            .await?;
        let info = self.get_token_info(chain_id, address, metrics).await?;

        let response = PriceResponseBody {
            fungibles: vec![FungiblePriceItem {
                address: format!(
                    "{}:{}:{}",
                    crypto::CaipNamespaces::Eip155,
                    chain_id,
                    address
                ),
                name: info.name,
                symbol: info.symbol,
                icon_url: info.logo_uri.unwrap_or_default(),
                price: price.parse().unwrap_or(0.0),
                decimals: info.decimals,
            }],
        };

        Ok(response)
    }
}
