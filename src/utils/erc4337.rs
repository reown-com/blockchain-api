use {
    alloy::{
        primitives::{Address, U256},
        rpc::client::{ClientBuilder, RpcClient},
        transports::TransportResult,
    },
    serde::{Deserialize, Serialize},
    url::Url,
    yttrium::user_operation::UserOperationV07,
};

pub struct BundlerRpcClient {
    pub client: RpcClient,
}

impl BundlerRpcClient {
    pub fn new(url: Url) -> Self {
        let client = ClientBuilder::default().http(url);
        Self { client }
    }

    pub async fn eth_estimate_user_operation_gas_v07(
        &self,
        user_op: &UserOperationV07,
        entrypoint: Address,
    ) -> TransportResult<EthEstimateUserOperationGasV07Response> {
        self.client
            .request("eth_estimateUserOperationGas", (user_op, entrypoint))
            .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthEstimateUserOperationGasV07Response {
    pub pre_verification_gas: U256,
    pub verification_gas_limit: U256,
    pub call_gas_limit: U256,
}
