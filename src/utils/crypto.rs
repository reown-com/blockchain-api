use {
    crate::{analytics::MessageSource, error::RpcError},
    alloy::{primitives::Address, rpc::json_rpc::Id},
    base64::prelude::*,
    bs58,
    ethers::{
        abi::Token,
        core::{
            k256::ecdsa::{signature::Verifier, Signature, VerifyingKey},
            types::Signature as EthSignature,
        },
        prelude::{abigen, EthAbiCodec, EthAbiType},
        providers::{Http, Middleware, Provider},
        types::{Address as EthersAddress, Bytes, H160, H256, U128, U256},
        utils::keccak256,
    },
    once_cell::sync::Lazy,
    regex::Regex,
    relay_rpc::auth::cacao::{signature::eip6492::verify_eip6492, CacaoError},
    serde::{Deserialize, Serialize},
    std::{str::FromStr, sync::Arc},
    strum::IntoEnumIterator,
    strum_macros::{Display, EnumIter, EnumString},
    tracing::{error, warn},
    url::Url,
};

const ENSIP11_MAINNET_COIN_TYPE: u32 = 60;
static CAIP_CHAIN_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[-a-zA-Z0-9]{1,32}").expect("Failed to initialize regexp for the chain ID format")
});
static CAIP_ETH_ADDRESS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"0x[a-fA-F0-9]{40}")
        .expect("Failed to initialize regexp for the eth address format")
});
static CAIP_SOLANA_ADDRESS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[1-9A-HJ-NP-Za-km-z]{32,44}")
        .expect("Failed to initialize regexp for the solana address format")
});

pub const SOLANA_NATIVE_TOKEN_ADDRESS: &str = "So11111111111111111111111111111111111111111";

pub const JSON_RPC_VERSION_STR: &str = "2.0";
pub static JSON_RPC_VERSION: once_cell::sync::Lazy<Arc<str>> =
    once_cell::sync::Lazy::new(|| Arc::from(JSON_RPC_VERSION_STR));

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
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("No result JSON-RPC call response")]
    NoResultInRpcResponse,
    #[error("Error in JSON-RPC call to the Bundler: {0}")]
    BundlerRpcResponseError(String),
}

/// JSON-RPC request schema
#[derive(Serialize, Clone, Debug)]
pub struct JsonRpcRequest<T: Serialize + Send + Sync> {
    pub id: Id,
    pub jsonrpc: Arc<str>,
    pub method: Arc<str>,
    pub params: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct BundlerJsonRpcParams {
    user_op: UserOperation,
    entry_point: String,
}

/// ERC-4337 bundler userOperation schema v0.7
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperation {
    pub sender: EthersAddress,
    /// The first 192 bits are the nonce key, the last 64 bits are the nonce value
    pub nonce: U256,
    pub call_data: Bytes,
    pub call_gas_limit: U128,
    pub verification_gas_limit: U128,
    pub pre_verification_gas: U256,
    pub max_priority_fee_per_gas: U128,
    pub max_fee_per_gas: U128,
    pub signature: Bytes,
    /*
     * Optional fields
     */
    /// Factory and data, are populated if deploying a new sender contract
    pub factory: Option<EthersAddress>,
    pub factory_data: Option<Bytes>,
    /// Paymaster and related fields are populated if using a paymaster
    pub paymaster: Option<EthersAddress>,
    pub paymaster_verification_gas_limit: Option<U128>,
    pub paymaster_post_op_gas_limit: Option<U128>,
    pub paymaster_data: Option<Bytes>,
}

impl UserOperation {
    /// Create a packed UserOperation v07 structure
    pub fn get_packed(&self) -> PackedUserOperation {
        let init_code = match (self.factory, self.factory_data.as_ref()) {
            (Some(factory), Some(factory_data)) => {
                let mut init_code = factory.as_bytes().to_vec();
                init_code.extend_from_slice(factory_data);
                Bytes::from(init_code)
            }
            _ => Bytes::new(),
        };

        let account_gas_limits = concat_128(
            self.verification_gas_limit.low_u128().to_be_bytes(),
            self.call_gas_limit.low_u128().to_be_bytes(),
        );

        let gas_fees = concat_128(
            self.max_priority_fee_per_gas.low_u128().to_be_bytes(),
            self.max_fee_per_gas.low_u128().to_be_bytes(),
        );

        let paymaster_and_data = match (
            self.paymaster,
            self.paymaster_verification_gas_limit,
            self.paymaster_post_op_gas_limit,
            self.paymaster_data.as_ref(),
        ) {
            (
                Some(paymaster),
                Some(paymaster_verification_gas_limit),
                Some(paymaster_post_op_gas_limit),
                Some(paymaster_data),
            ) => {
                let mut paymaster_and_data = paymaster.as_bytes().to_vec();
                paymaster_and_data
                    .extend_from_slice(&paymaster_verification_gas_limit.low_u128().to_be_bytes());
                paymaster_and_data
                    .extend_from_slice(&paymaster_post_op_gas_limit.low_u128().to_be_bytes());
                paymaster_and_data.extend_from_slice(paymaster_data);
                Bytes::from(paymaster_and_data)
            }
            _ => Bytes::new(),
        };

        PackedUserOperation {
            sender: self.sender,
            nonce: self.nonce,
            init_code,
            call_data: self.call_data.clone(),
            account_gas_limits: H256::from_slice(&account_gas_limits),
            pre_verification_gas: self.pre_verification_gas,
            gas_fees: H256::from_slice(&gas_fees),
            paymaster_and_data,
            signature: self.signature.clone(),
        }
    }
}

/// ERC-4337 bundler Packed userOperation schema for v07
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EthAbiCodec, EthAbiType)]
#[serde(rename_all = "camelCase")]
pub struct PackedUserOperation {
    pub sender: EthersAddress,
    pub nonce: U256,
    pub init_code: Bytes,
    pub call_data: Bytes,
    pub account_gas_limits: H256,
    pub pre_verification_gas: U256,
    pub gas_fees: H256,
    pub paymaster_and_data: Bytes,
    pub signature: Bytes,
}

fn concat_128(a: [u8; 16], b: [u8; 16]) -> [u8; 32] {
    std::array::from_fn(|i| {
        if let Some(i) = i.checked_sub(a.len()) {
            b[i]
        } else {
            a[i]
        }
    })
}

/// Convert message to EIP-191 compatible format
pub fn to_eip191_message(message: &[u8]) -> Vec<u8> {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut eip191_message = Vec::with_capacity(prefix.len() + message.len());
    eip191_message.extend_from_slice(prefix.as_bytes());
    eip191_message.extend_from_slice(message);
    eip191_message
}

/// Pack signature into a single byte array to Ethereum compatible format
pub fn pack_signature(unpacked: &EthSignature) -> Bytes {
    // Extract r, s, and v from the signature
    let r = unpacked.r;
    let s = unpacked.s;
    let v = if unpacked.v == 27 { 0x1b } else { 0x1c };
    let mut r_bytes = [0u8; 32];
    let mut s_bytes = [0u8; 32];
    r.to_big_endian(&mut r_bytes);
    s.to_big_endian(&mut s_bytes);
    // Pack r, s, and v into a single byte array
    let mut packed_signature = Vec::with_capacity(r_bytes.len() + s_bytes.len() + 1);
    packed_signature.extend_from_slice(&r_bytes);
    packed_signature.extend_from_slice(&s_bytes);
    packed_signature.push(v);
    Bytes::from(packed_signature)
}

/// Encode two bytes array into a single ABI encoded bytes
pub fn abi_encode_two_bytes_arrays(bytes1: &Bytes, bytes2: &Bytes) -> Bytes {
    let data = vec![Token::Bytes(bytes1.to_vec()), Token::Bytes(bytes2.to_vec())];
    Bytes::from(ethers::abi::encode(&[Token::Array(data)]))
}

/// Returns the keccak256 EIP-191 hash of the message
pub fn get_message_hash(message: &str) -> H256 {
    let prefixed_message = to_eip191_message(message.as_bytes());
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
#[tracing::instrument(level = "debug")]
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

/// Verify secp256k1 message signature using the verification key
/// Verification key is expected to be in DER format and Base64 encoded same as signature
#[tracing::instrument(level = "debug")]
pub fn verify_secp256k1_signature(
    message: &str,
    signature: &str,
    verification_key: &str,
) -> Result<(), RpcError> {
    let verifying_key = VerifyingKey::from_sec1_bytes(
        &BASE64_STANDARD
            .decode(verification_key)
            .map_err(|e| RpcError::WrongBase64Format(e.to_string()))?,
    )
    .map_err(|e| RpcError::KeyFormatError(e.to_string()))?;

    let signature_bytes = &BASE64_STANDARD
        .decode(signature)
        .map_err(|e| RpcError::WrongBase64Format(e.to_string()))?;
    let signature = Signature::from_der(signature_bytes)
        .map_err(|e| RpcError::SignatureFormatError(e.to_string()))?;

    let message_hash = keccak256(message.as_bytes());

    verifying_key
        .verify(&message_hash, &signature)
        .map_err(|e| RpcError::SignatureValidationError(e.to_string()))?;

    Ok(())
}

/// Get the balance of the ERC20 token
#[tracing::instrument(level = "debug")]
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
#[tracing::instrument(level = "debug")]
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
#[tracing::instrument(level = "debug")]
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

/// Call entry point v07 getUserOpHash contract and get the userOperation hash
#[tracing::instrument(level = "debug")]
pub async fn call_get_user_op_hash(
    rpc_project_id: &str,
    chain_id: &str,
    contract_address: H160,
    user_operation: UserOperation,
) -> Result<[u8; 32], CryptoUitlsError> {
    abigen!(
        EntryPoint,
        r#"[
            struct v07UserOperation { address sender; uint256 nonce; bytes initCode; bytes callData; bytes32 accountGasLimits; uint256 preVerificationGas; bytes32 gasFees; bytes paymasterAndData; bytes signature}
            function getUserOpHash(v07UserOperation calldata userOp) public view returns (bytes32)
        ]"#,
    );

    let provider = Provider::<Http>::try_from(format!(
        "https://rpc.walletconnect.com/v1?chainId={}&projectId={}",
        chain_id, rpc_project_id
    ))
    .map_err(|e| CryptoUitlsError::RpcUrlParseError(format!("Failed to parse RPC url: {}", e)))?;
    let provider = Arc::new(provider);

    let contract = EntryPoint::new(contract_address, provider);

    let packed_user_op = user_operation.get_packed();
    let user_op = v07UserOperation {
        sender: packed_user_op.sender,
        nonce: packed_user_op.nonce,
        init_code: packed_user_op.init_code,
        call_data: packed_user_op.call_data,
        account_gas_limits: packed_user_op.account_gas_limits.into(),
        pre_verification_gas: packed_user_op.pre_verification_gas,
        gas_fees: packed_user_op.gas_fees.into(),
        paymaster_and_data: packed_user_op.paymaster_and_data,
        signature: packed_user_op.signature,
    };

    let hash = contract
        .get_user_op_hash(user_op)
        .call()
        .await
        .map_err(|e| {
            CryptoUitlsError::ContractCallError(format!(
                "Failed to call getUserOpHash in EntryPoint contract: {}",
                e
            ))
        })?;

    Ok(hash)
}

/// Convert EVM chain ID to coin type ENSIP-11
#[tracing::instrument(level = "debug")]
pub fn convert_evm_chain_id_to_coin_type(chain_id: u32) -> u32 {
    // Exemption for the mainnet in ENSIP-11 format
    if chain_id == 1 {
        return ENSIP11_MAINNET_COIN_TYPE;
    }

    0x80000000 | chain_id
}

/// Convert coin type ENSIP-11 to EVM chain ID
#[tracing::instrument(level = "debug")]
pub fn convert_coin_type_to_evm_chain_id(coin_type: u32) -> u32 {
    // Exemption for the mainnet in ENSIP-11 format
    if coin_type == ENSIP11_MAINNET_COIN_TYPE {
        return 1;
    }

    0x7FFFFFFF & coin_type
}

/// Check if the coin type is in the supported list
#[tracing::instrument(level = "debug")]
pub fn is_coin_type_supported(coin_type: u32) -> bool {
    let evm_chain_id = convert_coin_type_to_evm_chain_id(coin_type);
    ChainId::iter().any(|x| x as u64 == evm_chain_id as u64)
}

/// Check if the address is in correct format
pub fn is_address_valid(address: &str, namespace: &CaipNamespaces) -> bool {
    match namespace {
        CaipNamespaces::Eip155 => {
            if !CAIP_ETH_ADDRESS_REGEX.is_match(address) {
                return false;
            }
            H160::from_str(address).is_ok()
        }
        CaipNamespaces::Solana => {
            if !CAIP_SOLANA_ADDRESS_REGEX.is_match(address) {
                return false;
            }
            match bs58::decode(address).into_vec() {
                Ok(decoded) => decoded.len() == 32,
                Err(_) => false,
            }
        }
    }
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
    #[strum(serialize = "base_sepolia_testnet", serialize = "base-sepolia-testnet")]
    BaseSepoliaTestnet = 84532,
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
    #[strum(
        serialize = "zksync",
        serialize = "zksyncera",
        serialize = "zksync-era"
    )]
    ZkSyncEra = 324,
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

    /// Is ChainID is supported
    pub fn is_supported(chain_id: u64) -> bool {
        ChainId::iter().any(|x| x as u64 == chain_id)
    }
}

#[derive(Clone, Copy, Debug, EnumString, EnumIter, Display, Eq, PartialEq, Deserialize, Hash)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CaipNamespaces {
    Eip155,
    Solana,
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
    if !is_address_valid(&address, &namespace) {
        return Err(CryptoUitlsError::WrongAddressFormat(address.clone()));
    };

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
    use {
        super::*,
        ethers::{
            core::k256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
            utils::keccak256,
        },
        rand_core::OsRng,
        std::collections::HashMap,
    };

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
        chains.insert("base_sepolia_testnet", "eip155:84532");

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
        chains.insert("eip155:84532", "base-sepolia-testnet");

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
        let caip10 = "eip155:1:0x1234567890123456789012345678901234567890";
        let result = disassemble_caip10(caip10).unwrap();
        assert_eq!(result.0, CaipNamespaces::Eip155);
        assert_eq!(result.1, "1".to_string());
        assert_eq!(
            result.2,
            "0x1234567890123456789012345678901234567890".to_string()
        );

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

    #[test]
    fn test_verify_secp256k1_signature() {
        let message = "test message";

        // Generate secp256k1 key pair
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        let public_key_der = verifying_key.to_encoded_point(false).as_bytes().to_vec();
        let public_key_der_base64 = BASE64_STANDARD.encode(public_key_der);

        // Hash the message using Keccak-256
        let message_hash = keccak256(message.as_bytes());

        // Sign the hashed message
        let signature: Signature = signing_key.sign(&message_hash);
        let signature_base64 = BASE64_STANDARD.encode(signature.to_der().as_bytes());

        // Correct signature and message
        assert!(
            verify_secp256k1_signature(message, &signature_base64, &public_key_der_base64).is_ok()
        );

        // Incorrect message
        assert!(verify_secp256k1_signature(
            "wrong message signature",
            &signature_base64,
            &public_key_der_base64
        )
        .is_err());
    }

    #[test]
    fn test_is_address_valid() {
        let valid_eth_address = "0x1234567890123456789012345678901234567890";
        let valid_sol_address = "CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo";
        let invalid_address = "67890123456789012340123456";

        assert!(is_address_valid(valid_eth_address, &CaipNamespaces::Eip155));
        assert!(!is_address_valid(invalid_address, &CaipNamespaces::Eip155));

        assert!(is_address_valid(valid_sol_address, &CaipNamespaces::Solana));
        assert!(!is_address_valid(invalid_address, &CaipNamespaces::Solana));
    }

    // Ignoring this test until the RPC project ID is provided by the CI workflow
    // The test can be run manually by providing the project ID
    #[ignore]
    #[tokio::test]
    async fn test_call_get_user_op_hash() {
        let rpc_project_id = ""; // Fill the project ID
        let chain_id = "eip155:11155111";
        // Entrypoint v07 contract address
        let contract_address = "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
            .parse::<H160>()
            .unwrap();
        // Dummy sender address
        let sender_address = "0x1234567890123456789012345678901234567890"
            .parse::<H160>()
            .unwrap();
        // Dummy user operation
        let user_op = UserOperation {
            sender: sender_address,
            nonce: U256::zero(),
            call_data: Bytes::from(vec![0x04, 0x05, 0x06]),
            call_gas_limit: U128::zero(),
            verification_gas_limit: U128::zero(),
            pre_verification_gas: U256::zero(),
            max_fee_per_gas: U128::zero(),
            max_priority_fee_per_gas: U128::zero(),
            signature: Bytes::from(vec![0x0a, 0x0b, 0x0c]),
            factory: None,
            factory_data: None,
            paymaster: None,
            paymaster_data: None,
            paymaster_post_op_gas_limit: None,
            paymaster_verification_gas_limit: None,
        };

        let result = call_get_user_op_hash(rpc_project_id, chain_id, contract_address, user_op)
            .await
            .unwrap();

        assert_eq!(
            hex::encode(result),
            "a5e787e98d421a0e62b2457e525bc8a4b1bde14cc71d48c0cf139b0b1fadb1cc"
        );
    }
}
