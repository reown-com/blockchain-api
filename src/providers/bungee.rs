use {
    crate::{
        error::RpcError,
        providers::{ChainOrchestrationProvider, ProviderKind},
        utils::crypto::disassemble_caip2,
    },
    alloy::primitives::{Address, U256},
    async_trait::async_trait,
    reqwest::Url,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    tracing::error,
};

pub const BRIDGING_SLIPPAGE: i8 = 3;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BungeeResponse<T> {
    pub result: T,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BungeeQuotes {
    pub routes: Vec<Value>,
    pub bridge_route_errors: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BungeeBuildTxRequest {
    pub route: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BungeeBuildTx {
    pub chain_id: usize,
    pub tx_data: String,
    pub tx_target: Address,
    pub value: String,
    pub approval_data: Option<BungeeApprovalData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BungeeApprovalData {
    pub allowance_target: Address,
    pub approval_token_address: Address,
    pub minimum_approval_amount: U256,
    pub owner: Address,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BungeeApprovalTx {
    pub from: Address,
    pub to: Address,
    pub data: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BungeeAllowance {
    pub value: U256,
}

#[derive(Debug)]
pub struct BungeeProvider {
    pub provider_kind: ProviderKind,
    pub api_key: String,
    pub base_api_url: String,
    pub http_client: reqwest::Client,
}

impl BungeeProvider {
    pub fn new(api_key: String) -> Self {
        let base_api_url = "https://api.socket.tech".to_string();
        let http_client = reqwest::Client::new();
        Self {
            provider_kind: ProviderKind::Bungee,
            api_key,
            base_api_url,
            http_client,
        }
    }

    async fn send_get_request(&self, url: Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("API-KEY", self.api_key.clone())
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
            .header("API-KEY", self.api_key.clone())
            .send()
            .await
    }
}

#[async_trait]
impl ChainOrchestrationProvider for BungeeProvider {
    #[tracing::instrument(skip(self), fields(provider = "Bungee"), level = "debug")]
    async fn get_bridging_quotes(
        &self,
        from_chain_id: String,
        from_token_address: Address,
        to_chain_id: String,
        to_token_address: Address,
        amount: U256,
        user_address: Address,
    ) -> Result<Vec<Value>, RpcError> {
        let mut url = Url::parse(format!("{}/v2/quote", &self.base_api_url).as_str())
            .map_err(|_| RpcError::ConversionParseURLError)?;

        let (_, evm_from_chain_id) = disassemble_caip2(&from_chain_id)?;
        let (_, evm_to_chain_id) = disassemble_caip2(&to_chain_id)?;

        url.query_pairs_mut()
            .append_pair("fromChainId", &evm_from_chain_id);
        url.query_pairs_mut()
            .append_pair("fromTokenAddress", &from_token_address.to_string());
        url.query_pairs_mut()
            .append_pair("toChainId", &evm_to_chain_id);
        url.query_pairs_mut()
            .append_pair("toTokenAddress", &to_token_address.to_string());
        url.query_pairs_mut()
            .append_pair("fromAmount", &amount.to_string());
        url.query_pairs_mut()
            .append_pair("userAddress", &user_address.to_string());
        url.query_pairs_mut()
            .append_pair("uniqueRoutesPerBridge", "true");
        url.query_pairs_mut().append_pair("sort", "output");
        url.query_pairs_mut().append_pair("singleTxOnly", "true");
        url.query_pairs_mut().append_pair(
            "defaultBridgeSlippage",
            BRIDGING_SLIPPAGE.to_string().as_str(),
        );
        // Use only Across bridge for latency reason
        url.query_pairs_mut()
            .append_pair("includeBridges", "across");

        let response = self.send_get_request(url).await?;
        if !response.status().is_success() {
            error!(
                "Failed to get bridging quotes from Bungee with status: {}",
                response.status()
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<BungeeResponse<BungeeQuotes>>().await?;
        if body.result.routes.is_empty() {
            error!(
                "No bridging routes available from Bungee provider. Bridges errors: {:?}",
                body.result.bridge_route_errors
            );
            return Ok(vec![]);
        }

        Ok(body.result.routes)
    }

    async fn build_bridging_tx(&self, route: Value) -> Result<BungeeBuildTx, RpcError> {
        let url = Url::parse(format!("{}/v2/build-tx", &self.base_api_url).as_str())
            .map_err(|_| RpcError::ConversionParseURLError)?;
        let response = self
            .send_post_request(url, &BungeeBuildTxRequest { route })
            .await?;
        if !response.status().is_success() {
            error!(
                "Failed to get bridging tx from Bungee with status: {}",
                response.status()
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<BungeeResponse<BungeeBuildTx>>().await?;

        Ok(body.result)
    }

    async fn check_allowance(
        &self,
        chain_id: String,
        owner: Address,
        target: Address,
        token_address: Address,
    ) -> Result<U256, RpcError> {
        let mut url =
            Url::parse(format!("{}/v2/approval/check-allowance", &self.base_api_url).as_str())
                .map_err(|_| RpcError::ConversionParseURLError)?;

        let (_, evm_chain_id) = disassemble_caip2(&chain_id)?;

        url.query_pairs_mut().append_pair("chainID", &evm_chain_id);
        url.query_pairs_mut()
            .append_pair("owner", &owner.to_string());
        url.query_pairs_mut()
            .append_pair("allowanceTarget", &target.to_string());
        url.query_pairs_mut()
            .append_pair("tokenAddress", &token_address.to_string());

        let response = self.send_get_request(url).await?;
        if !response.status().is_success() {
            error!(
                "Failed to get bridging allowance from Bungee with status: {}",
                response.status()
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<BungeeResponse<BungeeAllowance>>().await?;

        Ok(body.result.value)
    }

    async fn build_approval_tx(
        &self,
        chain_id: String,
        owner: Address,
        target: Address,
        token_address: Address,
        amount: U256,
    ) -> Result<BungeeApprovalTx, RpcError> {
        let mut url = Url::parse(format!("{}/v2/approval/build-tx", &self.base_api_url).as_str())
            .map_err(|_| RpcError::ConversionParseURLError)?;

        let (_, evm_chain_id) = disassemble_caip2(&chain_id)?;

        url.query_pairs_mut().append_pair("chainID", &evm_chain_id);
        url.query_pairs_mut()
            .append_pair("owner", &owner.to_string());
        url.query_pairs_mut()
            .append_pair("allowanceTarget", &target.to_string());
        url.query_pairs_mut()
            .append_pair("tokenAddress", &token_address.to_string());
        url.query_pairs_mut()
            .append_pair("amount", &amount.to_string());

        let response = self.send_get_request(url).await?;
        if !response.status().is_success() {
            error!(
                "Failed to get bridging approval tx from Bungee with status: {}",
                response.status()
            );
            return Err(RpcError::ConversionProviderError);
        }
        let body = response.json::<BungeeResponse<BungeeApprovalTx>>().await?;

        Ok(body.result)
    }
}
