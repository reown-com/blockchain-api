use {
    crate::{
        json_rpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcResult},
        metrics::Metrics,
        utils::crypto,
    },
    moka::future::Cache,
    strum_macros::{Display, EnumString},
    tracing::error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
pub enum CachedMethods {
    #[strum(serialize = "eth_chainId")]
    EthChainId,
}

/// Check if the response is cached and apply caching
pub async fn is_cached_response(
    caip2_chain_id: &str,
    request: &JsonRpcRequest,
    metrics: &Metrics,
    moka_cache: &Cache<String, String>,
) -> Option<JsonRpcResponse> {
    if let Ok(method) = request.method.as_ref().parse::<CachedMethods>() {
        match method {
            CachedMethods::EthChainId => {
                handle_eth_chain_id(caip2_chain_id, request, moka_cache, metrics).await
            }
        }
    } else {
        None
    }
}

fn construct_mem_cache_key(method: &str, caip2_chain_id: &str) -> String {
    format!("rpc_cache:{method}:{caip2_chain_id}")
}

async fn get_mem_cached_response(
    caip2_chain_id: &str,
    method: &str,
    moka_cache: &Cache<String, String>,
) -> Option<String> {
    moka_cache
        .get(&construct_mem_cache_key(method, caip2_chain_id))
        .await
}

async fn set_mem_cached_response(
    caip2_chain_id: &str,
    method: &str,
    value: &str,
    moka_cache: &Cache<String, String>,
) {
    moka_cache
        .insert(
            construct_mem_cache_key(method, caip2_chain_id),
            value.to_string(),
        )
        .await;
}

// Get bytes representation of the chain ID
// encoded as a hex‑string “quantity"
// https://eips.ethereum.org/EIPS/eip-155#chain-id
fn get_evm_chain_id_bytes(chain_id: &str) -> Option<String> {
    let chain_id_bytes = match chain_id.parse::<u64>() {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to parse chain_id {chain_id} as u64: {e}");
            return None;
        }
    };
    Some(format!("0x{chain_id_bytes:x}"))
}

async fn check_cached_chain_id_response(
    caip2_chain_id: &str,
    request: &JsonRpcRequest,
    moka_cache: &Cache<String, String>,
) -> Option<JsonRpcResponse> {
    get_mem_cached_response(
        caip2_chain_id,
        CachedMethods::EthChainId.to_string().as_str(),
        moka_cache,
    )
    .await
    .map(|cached_chain_id_bytes| {
        JsonRpcResponse::Result(JsonRpcResult::new(
            request.id.clone(),
            cached_chain_id_bytes.into(),
        ))
    })
}

/// Handle the eth_chainId RPC method caching
async fn handle_eth_chain_id(
    caip2_chain_id: &str,
    request: &JsonRpcRequest,
    moka_cache: &Cache<String, String>,
    metrics: &Metrics,
) -> Option<JsonRpcResponse> {
    let Ok((_, chain_id)) = crypto::disassemble_caip2(caip2_chain_id) else {
        error!("Failed to disassemble CAIP2 chainId: {caip2_chain_id}");
        return None;
    };

    // Check if the chainId is cached in the moka memory cache and return immediately
    if let Some(response) =
        check_cached_chain_id_response(caip2_chain_id, request, moka_cache).await
    {
        metrics.add_rpc_cached_call(
            caip2_chain_id.to_string(),
            CachedMethods::EthChainId.to_string(),
        );
        return Some(response);
    }

    let Some(chain_id_bytes) = get_evm_chain_id_bytes(&chain_id) else {
        error!("Failed to get chainId bytes for: {chain_id}");
        return None;
    };
    let response = JsonRpcResponse::Result(JsonRpcResult::new(
        request.id.clone(),
        chain_id_bytes.clone().into(),
    ));

    set_mem_cached_response(
        caip2_chain_id,
        CachedMethods::EthChainId.to_string().as_str(),
        &chain_id_bytes,
        moka_cache,
    )
    .await;
    metrics.add_rpc_cached_call(
        caip2_chain_id.to_string(),
        CachedMethods::EthChainId.to_string(),
    );
    Some(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_evm_chain_id_bytes() {
        assert_eq!(get_evm_chain_id_bytes("1"), Some("0x1".to_string()));
        assert_eq!(get_evm_chain_id_bytes("10"), Some("0xa".to_string()));
        assert_eq!(get_evm_chain_id_bytes("137"), Some("0x89".to_string()));
        assert_eq!(get_evm_chain_id_bytes("42161"), Some("0xa4b1".to_string()));
    }

    #[test]
    fn test_get_evm_chain_id_bytes_invalid() {
        assert_eq!(get_evm_chain_id_bytes("invalid"), None);
        assert_eq!(get_evm_chain_id_bytes(""), None);
        assert_eq!(get_evm_chain_id_bytes("abc"), None);
    }
}
