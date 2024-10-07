use {
    crate::{
        error::RpcResult,
        providers::{BundlerOpsProvider, SupportedBundlerOps},
        utils::crypto,
    },
    alloy::rpc::json_rpc::Id,
    async_trait::async_trait,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
    url::Url,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockAltoUrls {
    pub bundler_url: Url,
    pub paymaster_url: Url,
}

#[derive(Debug)]
pub struct MockAltoProvider {
    pub bundler_url: Url,
    pub paymaster_url: Url,
    http_client: reqwest::Client,
}

impl MockAltoProvider {
    pub fn new(override_bundler_urls: MockAltoUrls) -> Self {
        let http_client = reqwest::Client::new();
        Self {
            bundler_url: override_bundler_urls.bundler_url,
            paymaster_url: override_bundler_urls.paymaster_url,
            http_client,
        }
    }
}

#[async_trait]
impl BundlerOpsProvider for MockAltoProvider {
    async fn bundler_rpc_call(
        &self,
        _chain_id: &str,
        id: Id,
        jsonrpc: Arc<str>,
        method: &SupportedBundlerOps,
        params: serde_json::Value,
    ) -> RpcResult<serde_json::Value> {
        let jsonrpc_send_userop_request = crypto::JsonRpcRequest {
            id,
            jsonrpc,
            method: self.to_provider_op(method).into(),
            params,
        };
        let bundler_url = match method {
            SupportedBundlerOps::EthSendUserOperation
            | SupportedBundlerOps::EthGetUserOperationReceipt
            | SupportedBundlerOps::EthEstimateUserOperationGas
            | SupportedBundlerOps::PimlicoGetUserOperationGasPrice => self.bundler_url.clone(),
            SupportedBundlerOps::PmSponsorUserOperation => self.paymaster_url.clone(),
        };
        let response = self
            .http_client
            .post(bundler_url)
            .json(&jsonrpc_send_userop_request)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Check if there was an error in the response
        if let Some(error) = response.get("error") {
            return Err(
                crypto::CryptoUitlsError::BundlerRpcResponseError(error.to_string()).into(),
            );
        }
        Ok(response)
    }

    fn to_provider_op(&self, op: &SupportedBundlerOps) -> String {
        match op {
            SupportedBundlerOps::EthSendUserOperation => "eth_sendUserOperation".into(),
            SupportedBundlerOps::EthGetUserOperationReceipt => "eth_getUserOperationReceipt".into(),
            SupportedBundlerOps::EthEstimateUserOperationGas => {
                "eth_estimateUserOperationGas".into()
            }
            SupportedBundlerOps::PmSponsorUserOperation => "pm_sponsorUserOperation".into(),
            SupportedBundlerOps::PimlicoGetUserOperationGasPrice => {
                "pimlico_getUserOperationGasPrice".into()
            }
        }
    }
}
