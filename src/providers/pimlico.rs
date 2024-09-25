use {
    crate::{
        error::RpcResult,
        providers::{BundlerOpsProvider, SupportedBundlerOps},
        utils::crypto,
    },
    async_trait::async_trait,
};

#[derive(Debug)]
pub struct PimlicoProvider {
    pub api_key: String,
    pub base_api_url: String,
    http_client: reqwest::Client,
}

impl PimlicoProvider {
    pub fn new(api_key: String) -> Self {
        let base_api_url = "https://api.pimlico.io/v2".to_string();
        let http_client = reqwest::Client::new();
        Self {
            api_key,
            base_api_url,
            http_client,
        }
    }
}

#[async_trait]
impl BundlerOpsProvider for PimlicoProvider {
    async fn bundler_rpc_call(
        &self,
        chain_id: &str,
        id: u64,
        jsonrpc: &str,
        method: &SupportedBundlerOps,
        params: serde_json::Value,
    ) -> RpcResult<serde_json::Value> {
        let jsonrpc_send_userop_request = crypto::JsonRpcRequest {
            id,
            jsonrpc: jsonrpc.into(),
            method: self.to_provider_op(method).into(),
            params,
        };
        let bundler_url = format!(
            "{}/{}/rpc?apikey={}",
            self.base_api_url, chain_id, self.api_key
        );
        let response: serde_json::Value = self
            .http_client
            .post(bundler_url.clone())
            .json(&jsonrpc_send_userop_request)
            .send()
            .await?
            .json()
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
            SupportedBundlerOps::WalletGetCallsStatus => "wallet_getCallsStatus".into(),
            SupportedBundlerOps::WalletShowCallsStatus => "wallet_showCallsStatus".into(),
            SupportedBundlerOps::PmSponsorUserOperation => "pm_sponsorUserOperation".into(),
            SupportedBundlerOps::PimlicoGetUserOperationGasPrice => {
                "pimlico_getUserOperationGasPrice".into()
            }
        }
    }
}
