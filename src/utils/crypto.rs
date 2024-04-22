use {
    ethers::types::H160,
    std::str::FromStr,
    strum::IntoEnumIterator,
    strum_macros::{Display, EnumIter, EnumString},
    tracing::warn,
};

const ENSIP11_MAINNET_COIN_TYPE: u32 = 60;

#[derive(thiserror::Error, Debug)]
pub enum CryptoUitlsError {
    #[error("Namespace is not supported: {0}")]
    WrongNamespace(String),
    #[error("Wrong CAIP-2 format: {0}")]
    WrongCaip2Format(String),
    #[error("Wrong CAIP-10 format: {0}")]
    WrongCaip10Format(String),
}

/// Veryfy message signature signed by the keccak256
#[tracing::instrument]
pub fn verify_message_signature(
    message: &str,
    signature: &str,
    owner: &H160,
) -> Result<bool, Box<dyn std::error::Error>> {
    let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    let message_hash = ethers::core::utils::keccak256(prefixed_message.clone());
    let message_hash = ethers::types::H256::from_slice(&message_hash);

    let sign = ethers::types::Signature::from_str(signature)?;
    match sign.verify(message_hash, *owner) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
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
    let address = parts[2].to_string();
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

#[cfg(test)]
mod tests {
    use {
        super::*,
        ethers::types::H160,
        std::{collections::HashMap, str::FromStr},
    };

    #[test]
    fn test_verify_message_signature_valid() {
        let message = "test message signature";
        let signature = "0x660739ee06920c5f55fbaf0da4f435faaa9c55e2c9da303c50c4b3865191d67e5002a0b10eb0f89bae66823f7f07415ea9d5bbb607ee61ac98b7f2a0a44fcb5c1b";
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap();

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_message_signature_json() {
        let message = r#"{\"test\":\"some my text\"}"#;
        let signature = "0x2fe0b640b4036c9c97911e6f22c72a2c934f1d67db02948055c0e0c84dbf4f2b33c2f8c4b000642735dbf5d1c96ba48ccd2a998324c9e4cb7bb776f0c95ee2fc1b";
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap();

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        println!("result: {:?}", result);
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_message_signature_invalid() {
        let message = "wrong message signature";
        let signature = "0x660739ee06920c5f55fbaf0da4f435faaa9c55e2c9da303c50c4b3865191d67e5002a0b10eb0f89bae66823f7f07415ea9d5bbb607ee61ac98b7f2a0a44fcb5c1b"; // The signature of the message
        let owner = H160::from_str("0xAff392551773CCb2574fAE23195CC3aFDBe98d18").unwrap(); // The Ethereum address of the signer

        let result = verify_message_signature(message, signature, &owner);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

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
}
