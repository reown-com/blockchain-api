use {
    crate::{
        error::{RpcError, RpcResult},
        handlers::convert::{
            allowance::{AllowanceQueryParams, AllowanceResponseBody},
            approve::{
                ConvertApproveQueryParams,
                ConvertApproveResponseBody,
                ConvertApproveTx,
                ConvertApproveTxEip155,
            },
            gas_price::{GasPriceQueryParams, GasPriceQueryResponseBody},
            quotes::{ConvertQuoteQueryParams, ConvertQuoteResponseBody, QuoteItem},
            tokens::{TokenItem, TokensListQueryParams, TokensListResponseBody},
            transaction::{
                ConvertTransactionQueryParams,
                ConvertTransactionResponseBody,
                ConvertTx,
                ConvertTxEip155,
            },
        },
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
        let base_api_url = "https://api.1inch.dev".to_string();
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
    error: OneInchErrorItem,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneInchErrorItem {
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
    ) -> RpcResult<TokensListResponseBody> {
        let evm_chain_id = crypto::disassemble_caip2(&params.chain_id)?.1;
        let base = format!(
            "{}/swap/v6.0/{}/tokens",
            &self.base_api_url,
            evm_chain_id.clone()
        );
        let url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;

        let response = self.send_request(url, &self.http_client.clone()).await?;

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = response.json::<OneInchErrorResponse>().await?;
                return Err(RpcError::ConversionInvalidParameter(
                    response_error.error.description,
                ));
            }
            // 404 response is expected when the chain ID is not supported
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(RpcError::ConversionInvalidParameter(format!(
                    "Chain ID {} is not supported",
                    params.chain_id
                )));
            };

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

    #[tracing::instrument(skip(self), fields(provider = "1inch"), level = "debug")]
    async fn get_convert_quote(
        &self,
        params: ConvertQuoteQueryParams,
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

        url.query_pairs_mut().append_pair("src", &src_address);
        url.query_pairs_mut().append_pair("dst", &dst_address);
        url.query_pairs_mut().append_pair("amount", &params.amount);

        let response = self.send_request(url, &self.http_client.clone()).await?;

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = response.json::<OneInchErrorResponse>().await?;
                return Err(RpcError::ConversionInvalidParameter(
                    response_error.error.description,
                ));
            }

            error!(
                "Error on getting quotes for conversion from 1inch provider. Status is not OK: \
                 {:?}",
                response.status(),
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
            .append_pair("tokenAddress", &dst_address);
        if let Some(amount) = &params.amount {
            url.query_pairs_mut().append_pair("amount", amount);
        }

        let response = self.send_request(url, &self.http_client.clone()).await?;

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = response.json::<OneInchErrorResponse>().await?;
                return Err(RpcError::ConversionInvalidParameter(
                    response_error.error.description,
                ));
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

        url.query_pairs_mut().append_pair("src", &src_address);
        url.query_pairs_mut().append_pair("dst", &dst_address);
        url.query_pairs_mut().append_pair("amount", &params.amount);
        url.query_pairs_mut().append_pair("from", &user_address);

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

        let response = self.send_request(url, &self.http_client.clone()).await?;

        if !response.status().is_success() {
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = response.json::<OneInchErrorResponse>().await?;
                return Err(RpcError::ConversionInvalidParameter(
                    response_error.error.description,
                ));
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
                    gas: body.tx.gas.to_string(),
                    gas_price: body.tx.gas_price,
                }),
            },
        };

        Ok(response)
    }

    #[tracing::instrument(skip(self, params), fields(provider = "1inch"))]
    async fn get_gas_price(
        &self,
        params: GasPriceQueryParams,
    ) -> RpcResult<GasPriceQueryResponseBody> {
        let evm_chain_id = crypto::disassemble_caip2(&params.chain_id)?.1;
        let base = format!(
            "{}/gas-price/v1.5/{}",
            &self.base_api_url,
            evm_chain_id.clone()
        );
        let url = Url::parse(&base).map_err(|_| RpcError::ConversionParseURLError)?;

        let response = self.send_request(url, &self.http_client.clone()).await?;

        if !response.status().is_success() {
            error!(
                "Error on getting gas price for conversion from 1inch provider. Status is not OK: \
                 {:?}",
                response.status(),
            );
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = response.json::<OneInchErrorResponse>().await?;
                return Err(RpcError::ConversionInvalidParameter(
                    response_error.error.description,
                ));
            }
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
            .append_pair("tokenAddress", &token_address);
        url.query_pairs_mut()
            .append_pair("walletAddress", &wallet_address);

        let response = self.send_request(url, &self.http_client.clone()).await?;

        if !response.status().is_success() {
            error!(
                "Error on getting allowance for conversion from 1inch provider. Status is not OK: \
                 {:?}",
                response.status(),
            );
            // Passing through error description for the error context
            // if user parameter is invalid (got 400 status code from the provider)
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                let response_error = response.json::<OneInchErrorResponse>().await?;
                return Err(RpcError::ConversionInvalidParameter(
                    response_error.error.description,
                ));
            }
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<OneInchAllowanceResponse>().await?;
        Ok(AllowanceResponseBody {
            allowance: body.allowance,
        })
    }
}
