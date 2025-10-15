use {
    crate::{
        analytics::MessageSource,
        handlers::self_provider::SelfProviderPool,
        utils::crypto::{get_nonce, CryptoUitlsError},
    },
    alloy::primitives::{Address, U64},
    std::collections::HashMap,
    tokio::task::{JoinError, JoinHandle},
};

pub struct NonceManager {
    provider_pool: SelfProviderPool,
    nonce_manager: HashMap<(String, Address), NonceState>,
}

enum NonceState {
    Pending(JoinHandle<Result<U64, CryptoUitlsError>>),
    Ready(U64),
}

impl NonceManager {
    pub fn new(provider_pool: SelfProviderPool) -> Self {
        Self {
            provider_pool,
            nonce_manager: HashMap::new(),
        }
    }

    pub fn initialize_nonce(&mut self, chain_id: String, address: Address) {
        if !self
            .nonce_manager
            .contains_key(&(chain_id.clone(), address))
        {
            let provider = self
                .provider_pool
                .get_provider(chain_id.clone(), MessageSource::ChainAgnosticCheck);
            self.nonce_manager.insert(
                (chain_id, address),
                NonceState::Pending(tokio::task::spawn({
                    async move { get_nonce(address, &provider).await }
                })),
            );
        }
    }

    pub async fn get_nonce(
        &mut self,
        chain_id: String,
        address: Address,
    ) -> Result<Result<U64, CryptoUitlsError>, JoinError> {
        let nonce = self
            .nonce_manager
            .get_mut(&(chain_id, address))
            .expect("initialize_nonce must be called before get_nonce");
        let current_nonce = match nonce {
            NonceState::Pending(handle) => match handle.await {
                Ok(Ok(nonce)) => nonce,
                Ok(Err(e)) => return Ok(Err(e)),
                Err(e) => return Err(e),
            },
            NonceState::Ready(current_nonce) => *current_nonce,
        };
        *nonce = NonceState::Ready(current_nonce + U64::from(1));
        Ok(Ok(current_nonce))
    }
}
