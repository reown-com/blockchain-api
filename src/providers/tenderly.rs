use {
    crate::{error::RpcError, providers::SimulationProvider, utils::crypto::disassemble_caip2},
    alloy::primitives::{Address, Bytes, B256, U256},
    async_trait::async_trait,
    reqwest::Url,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    tracing::error,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimulationRequest {
    pub network_id: String,
    pub from: Address,
    pub to: Address,
    pub input: Bytes,
    pub estimate_gas: bool,
    pub state_objects: HashMap<Address, StateStorage>,
    pub save: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateStorage {
    pub storage: HashMap<B256, B256>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SimulationResponse {
    pub transaction: ResponseTransaction,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseTransaction {
    pub transaction_info: ResponseTransactionInfo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseTransactionInfo {
    pub asset_changes: Option<Vec<AssetChange>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AssetChange {
    #[serde(rename = "type")]
    pub asset_type: AssetChangeType,
    pub from: Address,
    pub to: Address,
    pub raw_amount: U256,
    pub token_info: TokenInfo,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum AssetChangeType {
    Transfer,
    Mint,
    Burn,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenInfo {
    pub standard: TokenStandard,
    pub contract_address: Address,
    pub decimals: u8,
    // TODO: Add more fields for the metadata
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TokenStandard {
    Erc20,
    Erc721,
}

#[derive(Debug)]
pub struct TenderlyProvider {
    pub api_key: String,
    pub account_slug: String,
    pub project_slug: String,
    pub base_api_url: String,
    pub http_client: reqwest::Client,
}

impl TenderlyProvider {
    pub fn new(api_key: String, account_slug: String, project_slug: String) -> Self {
        let base_api_url = format!(
            "https://api.tenderly.co/api/v1/account/{}/project/{}",
            account_slug, project_slug
        );
        let http_client = reqwest::Client::new();
        Self {
            api_key,
            account_slug,
            project_slug,
            base_api_url,
            http_client,
        }
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
            .header("X-Access-Key", self.api_key.clone())
            .send()
            .await
    }
}

#[async_trait]
impl SimulationProvider for TenderlyProvider {
    #[tracing::instrument(skip(self), fields(provider = "Tenderly"), level = "debug")]
    async fn simulate_transaction(
        &self,
        chain_id: String,
        from: Address,
        to: Address,
        input: Bytes,
        state_overrides: HashMap<Address, HashMap<B256, B256>>,
    ) -> Result<SimulationResponse, RpcError> {
        let url = Url::parse(format!("{}/simulate", &self.base_api_url).as_str())
            .map_err(|_| RpcError::ConversionParseURLError)?;
        let (_, evm_chain_id) = disassemble_caip2(&chain_id)?;

        // fill the state_objects with the state_overrides
        let mut state_objects: HashMap<Address, StateStorage> = HashMap::new();
        for (address, state) in state_overrides {
            let mut account_state = StateStorage {
                storage: HashMap::new(),
            };
            for (key, value) in state {
                account_state.storage.insert(key, value);
            }
            state_objects.insert(address, account_state);
        }

        let response = self
            .send_post_request(
                url,
                &SimulationRequest {
                    network_id: evm_chain_id,
                    from,
                    to,
                    input,
                    estimate_gas: true,
                    state_objects,
                    save: true,
                },
            )
            .await?;
        if !response.status().is_success() {
            error!(
                "Failed to get the transaction simulation response from Tenderly with status: {}",
                response.status()
            );
            return Err(RpcError::ConversionProviderError);
        }
        let response = response.json::<SimulationResponse>().await?;

        Ok(response)
    }
}
