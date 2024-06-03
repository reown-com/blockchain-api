use {
    crate::analytics::MessageSource,
    alloy_primitives::Address,
    ethers::{
        prelude::abigen,
        providers::{Http, Middleware, Provider},
        types::{H160, H256, U256},
    },
    once_cell::sync::Lazy,
    regex::Regex,
    relay_rpc::auth::cacao::{signature::eip6492::verify_eip6492, CacaoError},
    std::{str::FromStr, sync::Arc},
    strum::IntoEnumIterator,
    strum_macros::{Display, EnumIter, EnumString},
    tracing::warn,
    url::Url,
};

const ENSIP11_MAINNET_COIN_TYPE: u32 = 60;
static CAIP_CHAIN_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[-a-zA-Z0-9]{1,32}").expect("Failed to initialize regexp for the chain ID format")
});
static CAIP_ADDRESS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[-a-zA-Z0-9]{1,63}").expect("Failed to initialize regexp for the address format")
});

#[derive(thiserror::Error, Debug)]
pub enum CryptoUitlsError {
    #[error("Namespace is not supported: {0}")]
    WrongNamespace(String),
    #[error("Chain ID format is not supported: {0}")]
    WrongChainIdFormat(String),
    #[error("Address format is not supported: {0}")]
    WrongAddressFormat(String),
    #[error("Wrong CAIP-2 format: {0}")]
    WrongCaip2Format(String),
    #[error("Wrong CAIP-10 format: {0}")]
    WrongCaip10Format(String),
    #[error("Provider call error: {0}")]
    ProviderError(String),
    #[error("Contract call error: {0}")]
    ContractCallError(String),
    #[error("Wrong address format: {0}")]
    AddressFormat(String),
    #[error("Wrong signature format: {0}")]
    SignatureFormat(String),
    #[error("Wrong address checksum: {0}")]
    AddressChecksum(String),
    #[error("Failed to parse RPC url: {0}")]
    RpcUrlParseError(String),
}

pub fn add_eip191(message: &str) -> String {
    format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message)
}

/// Returns the keccak256 EIP-191 hash of the message
pub fn get_message_hash(message: &str) -> H256 {
    let prefixed_message = add_eip191(message);
    let message_hash = ethers::core::utils::keccak256(prefixed_message.clone());
    ethers::types::H256::from_slice(&message_hash)
}

pub async fn verify_message_signature(
    message: &str,
    signature: &str,
    address: &str,
    chain_id: &str,
    rpc_project_id: &str,
    source: MessageSource,
) -> Result<bool, CryptoUitlsError> {
    verify_eip6492_message_signature(
        message,
        signature,
        chain_id,
        address,
        rpc_project_id,
        source,
    )
    .await
}

/// Veryfy message signature for eip6492 contract
#[tracing::instrument]
pub async fn verify_eip6492_message_signature(
    message: &str,
    signature: &str,
    chain_id: &str,
    address: &str,
    rpc_project_id: &str,
    source: MessageSource,
) -> Result<bool, CryptoUitlsError> {
    let message_hash: [u8; 32] = get_message_hash(message).into();
    let address = Address::parse_checksummed(address, None)
        .map_err(|_| CryptoUitlsError::AddressChecksum(address.into()))?;

    let mut provider = Url::parse("https://rpc.walletconnect.com/v1")
        .map_err(|e| {
            CryptoUitlsError::RpcUrlParseError(format!(
                "Failed to parse RPC url:
        {}",
                e
            ))
        })
        .unwrap();
    provider.query_pairs_mut().append_pair("chainId", chain_id);
    provider
        .query_pairs_mut()
        .append_pair("projectId", rpc_project_id);
    provider
        .query_pairs_mut()
        .append_pair("source", &source.to_string());

    let hexed_signature = hex::decode(&signature[2..])
        .map_err(|e| CryptoUitlsError::SignatureFormat(format!("Wrong signature format: {}", e)))?;

    match verify_eip6492(hexed_signature, address, &message_hash, provider).await {
        Ok(_) => Ok(true),
        Err(CacaoError::Verification) => Ok(false),
        Err(e) => Err(CryptoUitlsError::ContractCallError(format!(
            "Failed to verify EIP-6492 signature: {}",
            e
        ))),
    }
}

/// Get the balance of the ERC20 token
#[tracing::instrument]
pub async fn get_erc20_balance(
    chain_id: &str,
    contract: H160,
    wallet: H160,
    rpc_project_id: &str,
    source: MessageSource,
) -> Result<U256, CryptoUitlsError> {
    // Use JSON-RPC call for the balance of the native ERC20 tokens
    // or call the contract for the custom ERC20 tokens
    let balance = if contract == H160::repeat_byte(0xee) {
        get_balance(chain_id, wallet, rpc_project_id, source).await?
    } else {
        get_erc20_contract_balance(chain_id, contract, wallet, rpc_project_id, source).await?
    };

    Ok(balance)
}

/// Get the balance of ERC20 token by calling the contract address
#[tracing::instrument]
async fn get_erc20_contract_balance(
    chain_id: &str,
    contract: H160,
    wallet: H160,
    rpc_project_id: &str,
    source: MessageSource,
) -> Result<U256, CryptoUitlsError> {
    abigen!(
        ERC20Contract,
        r#"[
            function balanceOf(address account) external view returns (uint256)
        ]"#,
    );

    let provider = Provider::<Http>::try_from(format!(
        "https://rpc.walletconnect.com/v1?chainId={}&projectId={}&source={}",
        chain_id, rpc_project_id, source
    ))
    .map_err(|e| CryptoUitlsError::RpcUrlParseError(format!("Failed to parse RPC url: {}", e)))?;
    let provider = Arc::new(provider);

    let contract = ERC20Contract::new(contract, provider);
    let balance = contract.balance_of(wallet).call().await.map_err(|e| {
        CryptoUitlsError::ContractCallError(format!(
            "Failed to call ERC20 contract for the balance: {}",
            e
        ))
    })?;
    Ok(balance)
}

/// Get the balance of the native coin
#[tracing::instrument]
async fn get_balance(
    chain_id: &str,
    wallet: H160,
    rpc_project_id: &str,
    source: MessageSource,
) -> Result<U256, CryptoUitlsError> {
    let provider = Provider::<Http>::try_from(format!(
        "https://rpc.walletconnect.com/v1?chainId={}&projectId={}&source={}",
        chain_id, rpc_project_id, source
    ))
    .map_err(|e| CryptoUitlsError::RpcUrlParseError(format!("Failed to parse RPC url: {}", e)))?;
    let provider = Arc::new(provider);

    let balance = provider
        .get_balance(wallet, None)
        .await
        .map_err(|e| CryptoUitlsError::ProviderError(format!("{}", e)))?;
    Ok(balance)
}

/// Convert EVM chain ID to coin type ENSIP-11
#[tracing::instrument]
pub fn convert_evm_chain_id_to_coin_type(chain_id: u32) -> u32 {
    // Exemption for the mainnet in ENSIP-11 format
    if chain_id == 1 {
        return ENSIP11_MAINNET_COIN_TYPE;
    }

    0x80000000 | chain_id
}

/// Convert coin type ENSIP-11 to EVM chain ID
#[tracing::instrument]
pub fn convert_coin_type_to_evm_chain_id(coin_type: u32) -> u32 {
    // Exemption for the mainnet in ENSIP-11 format
    if coin_type == ENSIP11_MAINNET_COIN_TYPE {
        return 1;
    }

    0x7FFFFFFF & coin_type
}

/// Check if the coin type is in the supported list
#[tracing::instrument]
pub fn is_coin_type_supported(coin_type: u32) -> bool {
    let evm_chain_id = convert_coin_type_to_evm_chain_id(coin_type);
    ChainId::iter().any(|x| x as u64 == evm_chain_id as u64)
}

/// Human readable chain ids to CAIP-2 chain ids
#[derive(Clone, Copy, Debug, EnumString, EnumIter, Display)]
#[strum(serialize_all = "lowercase")]
#[repr(u64)]
pub enum ChainId {
    Arbitrum = 42161,
    Aurora = 1313161554,
    Avalanche = 43114,
    Base = 8453,
    #[strum(
        to_string = "binance-smart-chain",
        serialize = "binance_smart_chain",
        serialize = "bsc"
    )]
    BinanceSmartChain = 56,
    Blast = 81032,
    Celo = 42220,
    #[strum(serialize = "ethereum", serialize = "mainnet")]
    Ethereum = 1,
    Fantom = 250,
    Goerli = 5,
    Linea = 59160,
    Optimism = 10,
    Polygon = 137,
    Scroll = 8508132,
    Sepolia = 11155111,
    #[strum(
        to_string = "xdai",
        serialize = "gnosis",
        serialize = "gnosis_chain",
        serialize = "gnosis-chain",
        serialize = "gnosischain"
    )]
    GnosisChain = 100,
    #[strum(serialize = "zksync", serialize = "zksyncera")]
    ZkSyncEra = 328,
    Zora = 7854577,
}

impl ChainId {
    /// Convert from human readable chain name id (e.g. polygon) to CAIP-2
    /// format chain id (e.g. `eip155:137`)
    pub fn to_caip2(chain_name: &str) -> Option<String> {
        match ChainId::from_str(chain_name) {
            Ok(chain_id) => Some(format!("eip155:{}", chain_id as u64)),
            Err(_) => {
                warn!("CAIP-2 Convertion: Chain name is not found: {}", chain_name);
                None
            }
        }
    }

    /// Convert from CAIP-2 format (e.g. `eip155:137`) to human readable chain
    /// name id (e.g. polygon)
    pub fn from_caip2(caip2_chain_id: &str) -> Option<String> {
        let extracted_chain_id = caip2_chain_id
            .split(':')
            .collect::<Vec<&str>>()
            .pop()
            .unwrap_or_default()
            .parse::<u64>()
            .unwrap_or_default();

        match ChainId::iter()
            .find(|&x| x as u64 == extracted_chain_id)
            .map(|x| x.to_string())
        {
            Some(chain_id) => Some(chain_id),
            None => {
                warn!(
                    "CAIP-2 Convertion: Chain id is not found: {}",
                    caip2_chain_id
                );
                None
            }
        }
    }
}

#[derive(Clone, Copy, Debug, EnumString, EnumIter, Display, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum CaipNamespaces {
    Eip155,
}

pub fn format_to_caip10(namespace: CaipNamespaces, chain_id: &str, address: &str) -> String {
    format!("{}:{}:{}", namespace, chain_id, address)
}

/// Disassemble CAIP-2 to namespace and chainId
pub fn disassemble_caip2(caip2: &str) -> Result<(CaipNamespaces, String), CryptoUitlsError> {
    let parts = caip2.split(':').collect::<Vec<&str>>();
    if parts.len() != 2 {
        return Err(CryptoUitlsError::WrongCaip2Format(caip2.into()));
    };
    let namespace = match parts.first() {
        Some(namespace) => match namespace.parse::<CaipNamespaces>() {
            Ok(namespace) => namespace,
            Err(_) => return Err(CryptoUitlsError::WrongNamespace(caip2.into())),
        },
        None => return Err(CryptoUitlsError::WrongNamespace(caip2.into())),
    };

    let chain_id = parts[1].to_string();
    CAIP_CHAIN_ID_REGEX
        .captures(&chain_id)
        .ok_or(CryptoUitlsError::WrongChainIdFormat(chain_id.clone()))?;
    Ok((namespace, chain_id))
}

/// Disassemble CAIP-10 to namespace, chainId and address
pub fn disassemble_caip10(
    caip10: &str,
) -> Result<(CaipNamespaces, String, String), CryptoUitlsError> {
    let parts = caip10.split(':').collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(CryptoUitlsError::WrongCaip10Format(caip10.into()));
    };
    let namespace = match parts.first() {
        Some(namespace) => match namespace.parse::<CaipNamespaces>() {
            Ok(namespace) => namespace,
            Err(_) => return Err(CryptoUitlsError::WrongNamespace(caip10.into())),
        },
        None => return Err(CryptoUitlsError::WrongNamespace(caip10.into())),
    };

    let chain_id = parts[1].to_string();
    CAIP_CHAIN_ID_REGEX
        .captures(&chain_id)
        .ok_or(CryptoUitlsError::WrongChainIdFormat(chain_id.clone()))?;

    let address = parts[2].to_string();
    CAIP_ADDRESS_REGEX
        .captures(&address)
        .ok_or(CryptoUitlsError::WrongAddressFormat(address.clone()))?;
    Ok((namespace, chain_id, address))
}

/// Compare two values (either H160 or &str) in constant time to prevent timing
/// attacks
pub fn constant_time_eq(a: impl AsRef<[u8]>, b: impl AsRef<[u8]>) -> bool {
    let a_bytes = a.as_ref();
    let b_bytes = b.as_ref();

    if a_bytes.len() != b_bytes.len() {
        return false;
    }

    let mut result = 0;
    for (byte_a, byte_b) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}

/// Format token amount to human readable format according to the token decimals
pub fn format_token_amount(amount: U256, decimals: u32) -> String {
    let amount_str = amount.to_string();
    let decimals_usize = decimals as usize;

    // Handle cases where the total digits are less than or equal to the decimals
    if amount_str.len() <= decimals_usize {
        let required_zeros = decimals_usize - amount_str.len();
        let zeros = "0".repeat(required_zeros);
        return format!("0.{}{}", zeros, amount_str);
    }

    // Insert the decimal point at the correct position
    let (integer_part, decimal_part) = amount_str.split_at(amount_str.len() - decimals_usize);
    format!("{}.{}", integer_part, decimal_part)
}

/// Convert token amount to value depending on the token price and decimals
pub fn convert_token_amount_to_value(balance: U256, price: f64, decimals: u32) -> f64 {
    let decimals_usize = decimals as usize;
    let scaling_factor = 10_u64.pow(decimals_usize as u32) as f64;
    let balance_f64 = balance.as_u64() as f64 / scaling_factor;
    balance_f64 * price
}

#[cfg(test)]
mod tests {
    use {super::*, std::collections::HashMap};

    #[test]
    fn test_convert_coin_type_to_evm_chain_id() {
        // Polygon
        let chain_id = 137;
        let coin_type = 2147483785;
        assert_eq!(convert_evm_chain_id_to_coin_type(chain_id), coin_type);
        assert_eq!(convert_coin_type_to_evm_chain_id(coin_type), chain_id);
    }

    #[test]
    fn test_is_coin_type_supported() {
        // Ethereum mainnet in ENSIP-11 format
        let coin_type_eth_mainnet = ENSIP11_MAINNET_COIN_TYPE;
        // Polygon in ENSIP-11 format
        let coin_type_polygon = 2147483785;
        // Not supported chain id
        let coin_type_not_supported = 2147483786;

        assert!(is_coin_type_supported(coin_type_eth_mainnet));
        assert!(is_coin_type_supported(coin_type_polygon));
        assert!(!is_coin_type_supported(coin_type_not_supported));
    }

    #[test]
    fn test_human_format_to_caip2_format() {
        let mut chains: HashMap<&str, &str> = HashMap::new();
        chains.insert("ethereum", "eip155:1");
        chains.insert("mainnet", "eip155:1");
        chains.insert("goerli", "eip155:5");
        chains.insert("optimism", "eip155:10");
        chains.insert("bsc", "eip155:56");
        chains.insert("gnosis", "eip155:100");
        chains.insert("xdai", "eip155:100");
        chains.insert("polygon", "eip155:137");
        chains.insert("base", "eip155:8453");

        for (chain_name, coin_type) in chains.iter() {
            let result = ChainId::to_caip2(chain_name);
            assert!(result.is_some(), "chain_name is not found: {}", chain_name);
            assert_eq!(&result.unwrap(), coin_type);
        }
    }

    #[test]
    fn test_caip2_format_to_human_format() {
        let mut chains: HashMap<&str, &str> = HashMap::new();
        chains.insert("eip155:1", "ethereum");
        chains.insert("eip155:5", "goerli");
        chains.insert("eip155:10", "optimism");
        chains.insert("eip155:56", "binance-smart-chain");
        chains.insert("eip155:100", "xdai");
        chains.insert("eip155:137", "polygon");
        chains.insert("eip155:8453", "base");

        for (chain_id, chain_name) in chains.iter() {
            let result = ChainId::from_caip2(chain_id);
            assert!(result.is_some(), "chain_id is not found: {}", chain_id);
            assert_eq!(&result.unwrap(), chain_name);
        }
    }

    #[test]
    fn test_constant_time_eq() {
        let string_one = "some string";
        let string_two = "some another string";
        assert!(!constant_time_eq(string_one, string_two));
        assert!(constant_time_eq(string_one, string_one));
    }

    #[test]
    fn test_format_to_caip10() {
        assert_eq!(
            format_to_caip10(CaipNamespaces::Eip155, "1", "0xtest"),
            "eip155:1:0xtest"
        );
    }

    #[test]
    fn test_disassemble_caip2() {
        let caip2 = "eip155:1";
        let result = disassemble_caip2(caip2).unwrap();
        assert_eq!(result.0, CaipNamespaces::Eip155);
        assert_eq!(result.1, "1".to_string());

        let malformed_caip2 = "eip1551";
        let error_result = disassemble_caip2(malformed_caip2);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_disassemble_caip10() {
        let caip10 = "eip155:1:0xtest";
        let result = disassemble_caip10(caip10).unwrap();
        assert_eq!(result.0, CaipNamespaces::Eip155);
        assert_eq!(result.1, "1".to_string());
        assert_eq!(result.2, "0xtest".to_string());

        let malformed_caip10 = "eip15510xtest";
        let error_result = disassemble_caip10(malformed_caip10);
        assert!(error_result.is_err());
    }

    #[test]
    fn test_format_token_amount() {
        // Test case for ethereum 18 decimals
        let amount_18 = U256::from_dec_str("959694527317077690").unwrap();
        let decimals_18 = 18;
        assert_eq!(
            format_token_amount(amount_18, decimals_18),
            "0.959694527317077690"
        );

        // Test case for polygon usdc 6 decimals
        let amount_6 = U256::from_dec_str("125320550").unwrap();
        let decimals_6 = 6;
        assert_eq!(format_token_amount(amount_6, decimals_6), "125.320550");
    }

    #[test]
    fn test_convert_token_amount_to_value() {
        let balance = U256::from_dec_str("959694527317077690").unwrap();
        let price = 10000.05;
        let decimals = 18;
        assert_eq!(
            convert_token_amount_to_value(balance, price, decimals),
            0.959_694_527_317_077_7 * price
        );
    }
}
