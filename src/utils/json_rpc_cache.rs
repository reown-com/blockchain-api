use {
    crate::{
        json_rpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcResult},
        metrics::Metrics,
        utils::crypto,
    },
    strum_macros::{Display, EnumString},
    tracing::error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
pub enum CachedMethods {
    #[strum(serialize = "eth_chainId")]
    EthChainId,
}

pub fn is_cached_response(
    caip2_chain_id: &str,
    request: &JsonRpcRequest,
    metrics: &Metrics,
) -> Option<JsonRpcResponse> {
    if let Ok(method) = request.method.as_ref().parse::<CachedMethods>() {
        match method {
            CachedMethods::EthChainId => {
                let Ok((_, chain_id)) = crypto::disassemble_caip2(caip2_chain_id) else {
                    tracing::error!("Failed to disassemble CAIP2 chainId: {caip2_chain_id}");
                    return None;
                };
                let Some(chain_id_bytes) = get_evm_chain_id_bytes(&chain_id) else {
                    error!("Failed to get chainId bytes for: {chain_id}");
                    return None;
                };
                let response = JsonRpcResponse::Result(JsonRpcResult::new(
                    request.id.clone(),
                    chain_id_bytes.into(),
                ));

                metrics.add_rpc_cached_call(caip2_chain_id.to_string(), method.to_string());
                Some(response)
            }
        }
    } else {
        None
    }
}

// Get bytes representation of the chainID
// by encoded as a hex‑string “quantity"
// https://eips.ethereum.org/EIPS/eip-155#chain-id
fn get_evm_chain_id_bytes(chain_id: &str) -> Option<String> {
    let chain_id_bytes = match chain_id.parse::<u64>() {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to parse chain_id {chain_id} as u64: {e}");
            return None;
        }
    };
    Some(format!("0x{chain_id_bytes:x}"))
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
