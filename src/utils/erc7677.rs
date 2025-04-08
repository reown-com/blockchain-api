use {
    alloy::{
        primitives::{Address, Bytes, U256, U64},
        rpc::client::{ClientBuilder, RpcClient},
        transports::TransportResult,
    },
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    url::Url,
    yttrium::user_operation::UserOperationV07,
};

pub struct PaymasterRpcClient {
    pub client: RpcClient,
}

impl PaymasterRpcClient {
    pub fn new(url: Url) -> Self {
        // TODO reuse reqwest client
        let client = ClientBuilder::default().http(url);
        Self { client }
    }

    pub async fn pm_get_paymaster_stub_data(
        &self,
        params: PmGetPaymasterDataParams,
    ) -> TransportResult<PmGetPaymasterStubDataResponse> {
        self.client
            .request("pm_getPaymasterStubData", params.into_tuple())
            .await
    }

    pub async fn pm_get_paymaster_data(
        &self,
        params: PmGetPaymasterDataParams,
    ) -> TransportResult<PmGetPaymasterStubDataResponse> {
        self.client
            .request("pm_getPaymasterData", params.into_tuple())
            .await
    }
}

pub struct PmGetPaymasterDataParams {
    pub user_op: UserOperationV07,
    pub entrypoint: Address,
    pub chain_id: U64,
    pub context: HashMap<String, serde_json::Value>,
}

impl PmGetPaymasterDataParams {
    pub fn into_tuple(
        self,
    ) -> (
        UserOperationV07,
        Address,
        U64,
        HashMap<String, serde_json::Value>,
    ) {
        (self.user_op, self.entrypoint, self.chain_id, self.context)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PmGetPaymasterStubDataResponse {
    pub paymaster: Address,
    pub paymaster_data: Bytes,
    pub paymaster_verification_gas_limit: U256,
    pub paymaster_post_op_gas_limit: U256,
    pub is_final: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PmGetPaymasterDataResponse {
    pub paymaster: Address,
    pub paymaster_data: Bytes,
}
