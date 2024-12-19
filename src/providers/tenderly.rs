use {
    crate::{
        error::RpcError,
        providers::{ProviderKind, SimulationProvider},
        storage::error::StorageError,
        utils::crypto::disassemble_caip2,
        Metrics,
    },
    alloy::primitives::{Address, Bytes, B256, U256},
    async_trait::async_trait,
    deadpool_redis::{redis::AsyncCommands, Pool},
    reqwest::Url,
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, sync::Arc, time::SystemTime},
    tracing::error,
};

// Caching TTL paramters
const ERC20_GAS_ESTIMATE_CACHE_TTL: u64 = 60 * 60 * 24; // 24 hours

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
    pub gas_used: u64,
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
    pub decimals: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TokenStandard {
    Erc20,
    Erc721,
}

pub struct TenderlyProvider {
    provider_kind: ProviderKind,
    api_key: String,
    base_api_url: String,
    http_client: reqwest::Client,
    redis_caching_pool: Option<Arc<Pool>>,
}

impl TenderlyProvider {
    pub fn new(
        api_key: String,
        account_slug: String,
        project_slug: String,
        redis_caching_pool: Option<Arc<Pool>>,
    ) -> Self {
        let base_api_url = format!(
            "https://api.tenderly.co/api/v1/account/{}/project/{}",
            account_slug, project_slug
        );
        let http_client = reqwest::Client::new();
        Self {
            provider_kind: ProviderKind::Tenderly,
            api_key,
            base_api_url,
            http_client,
            redis_caching_pool,
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

    /// Construct the cache key for the ERC20 gas estimate
    fn format_cache_erc20_gas_key(&self, chain_id: &str, contract_address: Address) -> String {
        format!("tenderly/erc20gas/{}/{}", chain_id, contract_address)
    }

    #[allow(dependency_on_unit_never_type_fallback)]
    async fn set_cache(&self, key: &str, value: &str, ttl: u64) -> Result<(), StorageError> {
        if let Some(redis_pool) = &self.redis_caching_pool {
            let mut cache = redis_pool.get().await.map_err(|e| {
                StorageError::Connection(format!("Error when getting the Redis pool instance {e}"))
            })?;
            cache
                .set_ex(key, value, ttl)
                .await
                .map_err(|e| StorageError::Connection(format!("Error when seting cache: {e}")))?;
        }
        Ok(())
    }

    #[allow(dependency_on_unit_never_type_fallback)]
    async fn get_cache(&self, key: &str) -> Result<Option<String>, StorageError> {
        if let Some(redis_pool) = &self.redis_caching_pool {
            let mut cache = redis_pool.get().await.map_err(|e| {
                StorageError::Connection(format!("Error when getting the Redis pool instance {e}"))
            })?;
            let value = cache
                .get(key)
                .await
                .map_err(|e| StorageError::Connection(format!("Error when getting cache: {e}")))?;
            return Ok(value);
        }
        Ok(None)
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
        metrics: Arc<Metrics>,
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

        let latency_start = SystemTime::now();
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
        metrics.add_latency_and_status_code_for_provider(
            self.provider_kind,
            response.status().into(),
            latency_start,
            None,
            Some("simulate".to_string()),
        );

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

    #[tracing::instrument(skip(self), fields(provider = "Tenderly"), level = "debug")]
    async fn get_cached_erc20_gas_estimation(
        &self,
        chain_id: &str,
        contract_address: Address,
    ) -> Result<Option<u64>, RpcError> {
        let cache_key = self.format_cache_erc20_gas_key(chain_id, contract_address);
        let cached_value = self.get_cache(&cache_key).await?;
        if let Some(value) = cached_value {
            return Ok(Some(value.parse().unwrap()));
        }
        Ok(None)
    }

    #[tracing::instrument(skip(self), fields(provider = "Tenderly"), level = "debug")]
    async fn set_cached_erc20_gas_estimation(
        &self,
        chain_id: &str,
        contract_address: Address,
        gas: u64,
    ) -> Result<(), RpcError> {
        let cache_key = self.format_cache_erc20_gas_key(chain_id, contract_address);
        self.set_cache(&cache_key, &gas.to_string(), ERC20_GAS_ESTIMATE_CACHE_TTL)
            .await?;
        Ok(())
    }
}