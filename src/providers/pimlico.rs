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
};

#[derive(Debug)]
pub struct PimlicoProvider {
    pub api_key: String,
    pub base_api_url: String,
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceOverride {
    balance: String,
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
        id: Id,
        jsonrpc: Arc<str>,
        method: &SupportedBundlerOps,
        mut params: serde_json::Value,
    ) -> RpcResult<serde_json::Value> {
        // Adding an exclusion for the `eth_estimateUserOperationGas`` to add the
        // balance override during estimation to prevent AA21 errors
        if method == &SupportedBundlerOps::EthEstimateUserOperationGas {
            if let Some(array) = params.as_array_mut() {
                // Apply the balance override injection only if the array
                // length is 2 parameters (no status override passed as a third parameter)
                if array.len() == 2 {
                    if let Some(sender) = array
                        .first()
                        .and_then(|first| first.get("sender"))
                        .and_then(serde_json::Value::as_str)
                    {
                        // Adding 100 ETH to the smart account
                        let new_param = serde_json::json!({
                            sender: BalanceOverride { balance: "0x56BC75E2D63100000".into() },
                        });
                        array.push(new_param);
                    }
                }
            }
        }
        let jsonrpc_send_userop_request = crypto::JsonRpcRequest {
            id,
            jsonrpc,
            method: self.to_provider_op(method).into(),
            params,
        };
        let bundler_url = format!(
            "{}/{}/rpc?apikey={}",
            self.base_api_url, chain_id, self.api_key
        );
        let response = self
            .http_client
            .post(bundler_url.clone())
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
            SupportedBundlerOps::PmGetPaymasterData => "pm_getPaymasterData".into(),
            SupportedBundlerOps::PmGetPaymasterStubData => "pm_getPaymasterStubData".into(),
            SupportedBundlerOps::PimlicoGetUserOperationGasPrice => {
                "pimlico_getUserOperationGasPrice".into()
            }
        }
    }
}
