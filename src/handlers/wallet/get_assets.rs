use crate::{
    error::RpcError,
    handlers::{
        self,
        balance::{BalanceItem, BalanceQuantity, BalanceQueryParams, BalanceResponseBody},
        SdkInfoParams, SupportedCurrencies, HANDLER_TASK_METRICS,
    },
    state::AppState,
};
use alloy::primitives::{address, Address, U64};
use axum::{
    extract::{ConnectInfo, Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::{HeaderMap, StatusCode};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use thiserror::Error;
use tracing::error;
use wc::future::FutureExt;

// https://github.com/ethereum/ERCs/pull/709/files#diff-be675f3ce6b6aa5616dd1bccf5e50f44ad65775afb967a47aaffb8f5eb51b849R35
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsParams {
    account: Address,
    #[serde(flatten)]
    filters: GetAssetsFilters,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsFilters {
    #[serde(default)]
    asset_filter: Option<AssetFilter>,
    #[serde(default)]
    asset_type_filter: Option<AssetTypeFilter>,
    #[serde(default)]
    chain_filter: Option<ChainFilter>,
}

pub type AssetFilter = HashMap<Eip155ChainId, Vec<AddressOrNative>>;
pub type AssetTypeFilter = Vec<AssetType>;
pub type ChainFilter = Vec<Eip155ChainId>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum AssetType {
    Native,
    Erc20,
    Erc721,
}

pub type Eip155ChainId = U64;
pub type GetAssetsResult = HashMap<Eip155ChainId, Vec<Asset>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum Asset {
    Native {
        #[serde(flatten)]
        data: AssetData<NativeMetadata>,
    },
    Erc20 {
        #[serde(flatten)]
        data: AssetData<Erc20Metadata>,
    },
    Erc721 {
        #[serde(flatten)]
        data: AssetData<Erc721Metadata>,
    },
}

impl Asset {
    pub fn balance(&self) -> U64 {
        match self {
            Self::Native { data } => data.balance,
            Self::Erc20 { data } => data.balance,
            Self::Erc721 { data } => data.balance,
        }
    }

    pub fn asset_type(&self) -> AssetType {
        match self {
            Self::Native { .. } => AssetType::Native,
            Self::Erc20 { .. } => AssetType::Erc20,
            Self::Erc721 { .. } => AssetType::Erc721,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressOrNative {
    Address(Address),
    Native,
}

impl AddressOrNative {
    pub fn as_address(&self) -> Option<&Address> {
        match self {
            Self::Address(address) => Some(address),
            Self::Native => None,
        }
    }
}

impl Serialize for AddressOrNative {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AddressOrNative::Native => serializer.serialize_str("native"),
            AddressOrNative::Address(address) => address.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for AddressOrNative {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the input as a string.
        let s = String::deserialize(deserializer)?;

        if s == "native" {
            Ok(AddressOrNative::Native)
        } else {
            s.parse::<Address>()
                .map_err(de::Error::custom)
                .map(AddressOrNative::Address)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetData<M> {
    address: AddressOrNative,
    balance: U64,
    metadata: M,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeMetadata {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc20Metadata {
    name: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc721Metadata {
    name: String,
    symbol: String,
}

#[derive(Error, Debug)]
pub enum GetAssetsError {
    #[error("Internal error")]
    InternalError(GetAssetsErrorInternalError),
}

#[derive(Error, Debug)]
pub enum GetAssetsErrorInternalError {
    #[error("GetBalance call failed: {0}")]
    GetBalance(RpcError),
}

impl IntoResponse for GetAssetsError {
    fn into_response(self) -> Response {
        #[allow(unreachable_patterns)] // TODO remove
        match self {
            Self::InternalError(e) => {
                error!("HTTP server error: (get_assets) {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            e => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": e.to_string(),
                })),
            )
                .into_response(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    project_id: String,
    request: GetAssetsParams,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
) -> Result<GetAssetsResult, GetAssetsError> {
    handler_internal(state, project_id, request, connect_info, headers, query)
        .with_metrics(HANDLER_TASK_METRICS.with_name("wallet_get_assets"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
async fn handler_internal(
    state: State<Arc<AppState>>,
    project_id: String,
    request: GetAssetsParams,
    ConnectInfo(connect_info): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<QueryParams>,
) -> Result<GetAssetsResult, GetAssetsError> {
    let balance = handlers::balance::handler(
        state,
        Query(BalanceQueryParams {
            project_id,
            currency: SupportedCurrencies::USD,
            chain_id: None,
            force_update: None,
            sdk_info: query.sdk_info.clone(),
        }),
        ConnectInfo(connect_info),
        headers,
        Path(request.account.to_string()),
    )
    .await
    .map_err(|e| GetAssetsError::InternalError(GetAssetsErrorInternalError::GetBalance(e)))?;

    get_assets(balance.0, request.filters)
}

fn get_assets(
    balance: BalanceResponseBody,
    filters: GetAssetsFilters,
) -> Result<GetAssetsResult, GetAssetsError> {
    let (to_aggregate_balance, not_to_aggregate_balance) = segregate_balances(balance);
    let aggregated_balances = apply_aggregate_balance_value(to_aggregate_balance);
    let balances_to_filter = aggregated_balances
        .into_iter()
        .chain(not_to_aggregate_balance)
        .collect::<Vec<_>>();
    let filtered_balances = filter_balances(balances_to_filter, filters);
    Ok(create_response(filtered_balances))
}

const CHAIN_ID_OPTIMISM: &str = "eip155:10";
const CHAIN_ID_BASE: &str = "eip155:8453";
const CHAIN_ID_ARBITRUM: &str = "eip155:42161";

const SUPPORTED_CHAINS: [&str; 3] = [CHAIN_ID_OPTIMISM, CHAIN_ID_BASE, CHAIN_ID_ARBITRUM];

fn get_erc20_groups() -> HashMap<&'static str, HashMap<&'static str, Address>> {
    HashMap::from([
        (
            "USDC",
            HashMap::from([
                (
                    CHAIN_ID_ARBITRUM,
                    address!("af88d065e77c8cC2239327C5EDb3A432268e5831"),
                ),
                (
                    CHAIN_ID_OPTIMISM,
                    address!("0b2C639c533813f4Aa9D7837CAf62653d097Ff85"),
                ),
                (
                    CHAIN_ID_BASE,
                    address!("833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
                ),
            ]),
        ),
        (
            "USDT",
            HashMap::from([
                (
                    CHAIN_ID_ARBITRUM,
                    address!("Fd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9"),
                ),
                (
                    CHAIN_ID_OPTIMISM,
                    address!("94b008aA00579c1307B0EF2c499aD98a8ce58e58"),
                ),
            ]),
        ),
    ])
}

fn get_supported_token_and_chain_pair_key(
    chain_id: &str,
    address: AddressOrNative,
) -> Option<&'static str> {
    for (key, chain_pairs) in get_erc20_groups() {
        if let Some(a) = chain_pairs.get(chain_id) {
            if let AddressOrNative::Address(ref address) = address {
                if a == address {
                    return Some(key);
                }
            }
        }
    }
    None
}

fn convert_caip10_to_address(caip10: &str) -> Address {
    caip10.split(":").last().unwrap().parse().unwrap()
}

fn group_balances(
    balances: Vec<BalanceItem>,
) -> (HashMap<&'static str, Vec<BalanceItem>>, Vec<BalanceItem>) {
    let mut to_aggregate_balance = HashMap::new();
    let mut not_to_aggregate_balance = Vec::new();

    for balance in balances {
        let address = balance
            .address
            .as_ref()
            .map(|a| convert_caip10_to_address(a.as_ref()))
            .map(AddressOrNative::Address)
            .unwrap_or(AddressOrNative::Native);

        let token_key =
            get_supported_token_and_chain_pair_key(balance.chain_id.as_ref().unwrap(), address);
        if let Some(token_key) = token_key {
            to_aggregate_balance
                .entry(token_key)
                .or_insert_with(Vec::new)
                .push(balance);
        } else {
            not_to_aggregate_balance.push(balance);
        }
    }

    (to_aggregate_balance, not_to_aggregate_balance)
}

fn fill_missing_chains(to_aggregate_balance: &mut HashMap<&'static str, Vec<BalanceItem>>) {
    let to_aggregate_balance_clone = to_aggregate_balance.clone();
    for (token_key, chain_pairs) in to_aggregate_balance.iter_mut() {
        if chain_pairs.len() != SUPPORTED_CHAINS.len() {
            let missing_chains = SUPPORTED_CHAINS.iter().filter(|chain| {
                !to_aggregate_balance_clone[token_key]
                    .iter()
                    .any(|b| b.chain_id.as_deref() == Some(*chain))
            });
            for chain in missing_chains {
                let template_balance = chain_pairs[0].clone();
                chain_pairs.push(BalanceItem {
                    chain_id: Some(chain.to_string()),
                    address: Some(format!(
                        "{}:{}",
                        chain,
                        get_erc20_groups()[token_key][chain]
                    )),
                    quantity: BalanceQuantity {
                        decimals: template_balance.quantity.decimals.clone(),
                        numeric: "0".to_owned(),
                    },
                    value: Some(0.0),
                    ..template_balance
                });
            }
        }
    }
}

fn segregate_balances(
    balance: BalanceResponseBody,
) -> (HashMap<&'static str, Vec<BalanceItem>>, Vec<BalanceItem>) {
    let (mut to_aggregate_balance, not_to_aggregate_balance) = group_balances(balance.balances);
    fill_missing_chains(&mut to_aggregate_balance);
    (to_aggregate_balance, not_to_aggregate_balance)
}

fn apply_aggregate_balance_value(
    to_aggregate_balance: HashMap<&'static str, Vec<BalanceItem>>,
) -> Vec<BalanceItem> {
    let mut aggregated_balances = Vec::new();

    for (_, balances) in to_aggregate_balance {
        if balances.is_empty() {
            continue;
        }

        let new_balances = balances
            .clone()
            .into_iter()
            .enumerate()
            .map(|(current_index, current_balance)| {
                let current_value = current_balance.quantity.numeric.parse::<f64>().unwrap();

                let mut highest_other_value = 0.0;
                for (index, balance) in balances.iter().enumerate() {
                    if index != current_index {
                        let value = balance.quantity.numeric.parse::<f64>().unwrap();
                        highest_other_value = value.max(highest_other_value);
                    }
                }

                let aggregated_value = current_value + highest_other_value;

                BalanceItem {
                    quantity: BalanceQuantity {
                        numeric: aggregated_value.to_string(),
                        ..current_balance.quantity
                    },
                    value: Some(current_balance.price * aggregated_value),
                    ..current_balance
                }
            })
            .collect::<Vec<_>>();

        aggregated_balances.extend(new_balances);
    }

    aggregated_balances
}

fn filter_balances(balances: Vec<BalanceItem>, filters: GetAssetsFilters) -> Vec<BalanceItem> {
    let mut balances = balances;
    if let Some(asset_filter) = filters.asset_filter {
        balances = apply_asset_filter(asset_filter, balances);

        // Early return since futher filters should be redundant?
        return sort_balances_by_value(balances);
    }

    if let Some(asset_type_filter) = filters.asset_type_filter {
        balances = apply_asset_type_filter(asset_type_filter, balances);
    }

    if let Some(chain_filter) = filters.chain_filter {
        balances = apply_chain_filter(chain_filter, balances);
    }

    sort_balances_by_value(balances)
}

fn apply_asset_filter(asset_filter: AssetFilter, balances: Vec<BalanceItem>) -> Vec<BalanceItem> {
    let mut filtered_balances = Vec::with_capacity(balances.len());

    for (chain, addresses) in asset_filter {
        for address in addresses {
            let new = balances.clone().into_iter().filter(|balance| {
                balance.chain_id == Some(format!("eip155:{chain}"))
                    && balance
                        .address
                        .as_ref()
                        .map(|a| convert_caip10_to_address(a.as_str()))
                        .map(AddressOrNative::Address)
                        .unwrap_or(AddressOrNative::Native)
                        == address
            });
            filtered_balances.extend(new);
        }
    }

    filtered_balances
}

fn apply_chain_filter(chain_filter: ChainFilter, balances: Vec<BalanceItem>) -> Vec<BalanceItem> {
    let chain_filter = chain_filter
        .into_iter()
        .map(|chain| Some(format!("eip155:{chain}")))
        .collect::<Vec<_>>();
    balances
        .into_iter()
        .filter(|balance| chain_filter.contains(&balance.chain_id))
        .collect()
}

fn apply_asset_type_filter(
    asset_type_filter: AssetTypeFilter,
    balances: Vec<BalanceItem>,
) -> Vec<BalanceItem> {
    balances
        .into_iter()
        .filter(|balance| {
            asset_type_filter.contains(
                &balance
                    .address
                    .as_ref()
                    .map(|_| AssetType::Erc20)
                    .unwrap_or(AssetType::Native),
            )
        })
        .collect()
}

fn sort_balances_by_value(mut balances: Vec<BalanceItem>) -> Vec<BalanceItem> {
    balances.sort_by(|b1, b2| b2.value.unwrap_or(0.).total_cmp(&b1.value.unwrap_or(0.)));
    balances
}

fn create_response(balances: Vec<BalanceItem>) -> GetAssetsResult {
    let mut result = HashMap::new();
    for balance in balances {
        result
            .entry(
                balance
                    .chain_id
                    .unwrap()
                    .strip_prefix("eip155:")
                    .unwrap()
                    .parse()
                    .unwrap(),
            )
            .or_insert_with(Vec::new)
            .push({
                fn convert_balance_to_hex(quantity: &BalanceQuantity) -> U64 {
                    U64::from(
                        (quantity.numeric.parse::<f64>().unwrap()
                            * 10f64.powf(quantity.decimals.parse::<f64>().unwrap()))
                        .round() as u64,
                    )
                }
                let asset_balance = convert_balance_to_hex(&balance.quantity);

                match balance.address {
                    Some(address) => Asset::Erc20 {
                        data: AssetData {
                            address: AddressOrNative::Address(convert_caip10_to_address(&address)),
                            balance: asset_balance,
                            metadata: Erc20Metadata {
                                name: balance.name,
                                symbol: balance.symbol,
                                decimals: balance.quantity.decimals.parse().unwrap(),
                                // TODO value
                                // TODO price
                                // TODO iconUrl
                            },
                        },
                    },
                    None => Asset::Native {
                        data: AssetData {
                            address: AddressOrNative::Native,
                            balance: asset_balance,
                            metadata: NativeMetadata {
                                // TODO name
                                // TODO symbol
                                // TODO decimals
                                // TODO value
                                // TODO price
                                // TODO iconUrl
                            },
                        },
                    },
                }
            });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_assets() {
        let balance = BalanceResponseBody { balances: vec![] };
        let assets = get_assets(
            balance,
            GetAssetsFilters {
                asset_filter: None,
                asset_type_filter: None,
                chain_filter: None,
            },
        )
        .unwrap();
        assert!(assets.is_empty());
    }
}

#[cfg(test)]
mod ported_tests {
    use super::*;
    use crate::handlers::balance::{BalanceItem, BalanceQuantity, BalanceResponseBody};
    use alloy::primitives::{address, Address};

    const _ACCOUNT: Address = address!("F91D77EcEA92261d8CfBD9B235709d6ff6233fae");

    fn mock_balance_response() -> BalanceResponseBody {
        BalanceResponseBody {
            balances: vec![
                BalanceItem {
                    name: "USDC".to_owned(),
                    symbol: "USDC".to_owned(),
                    chain_id: Some("eip155:8453".to_owned()),
                    address: Some(
                        "eip155:8453:0x833589fcd6edb6e08f4c7c32d4f71b54bda02913".to_owned(),
                    ),
                    value: Some(2.007645999867311),
                    price: 0.999999999933908,
                    quantity: BalanceQuantity {
                        decimals: "6".to_owned(),
                        numeric: "2.007646".to_owned(),
                    },
                    icon_url: "https://s2.coinmarketcap.com/static/img/coins/128x128/3408.png"
                        .to_owned(),
                },
                BalanceItem {
                    name: "Ethereum".to_owned(),
                    symbol: "ETH".to_owned(),
                    chain_id: Some("eip155:10".to_owned()),
                    address: None,
                    value: Some(0.8475147271862257),
                    price: 2772.310987,
                    quantity: BalanceQuantity {
                        decimals: "18".to_owned(),
                        numeric: "0.000305706946717167".to_owned(),
                    },
                    icon_url: "https://cdn.zerion.io/eth.png".to_owned(),
                },
                BalanceItem {
                    name: "Ethereum".to_owned(),
                    symbol: "ETH".to_owned(),
                    chain_id: Some("eip155:8453".to_owned()),
                    address: None,
                    value: Some(0.7866910412902113),
                    price: 2772.189181,
                    quantity: BalanceQuantity {
                        decimals: "18".to_owned(),
                        numeric: "0.000283779709798316".to_owned(),
                    },
                    icon_url: "https://cdn.zerion.io/eth.png".to_owned(),
                },
                BalanceItem {
                    name: "USDC".to_owned(),
                    symbol: "USDC".to_owned(),
                    chain_id: Some("eip155:10".to_owned()),
                    address: Some(
                        "eip155:10:0x0b2c639c533813f4aa9d7837caf62653d097ff85".to_owned(),
                    ),
                    value: Some(0.5476979998447937),
                    price: 0.9999999997166208,
                    quantity: BalanceQuantity {
                        decimals: "6".to_owned(),
                        numeric: "0.547698".to_owned(),
                    },
                    icon_url: "https://s2.coinmarketcap.com/static/img/coins/128x128/3408.png"
                        .to_owned(),
                },
            ],
        }
    }

    mod filtering {
        #[test]
        fn should_filter_assets_by_asset_filter_correctly() {
            // TODO
        }

        // TODO
    }

    mod aggregation_and_conversion {
        use super::*;

        #[test]
        fn should_correctly_convert_balance_to_hex() {
            let result = get_assets(
                mock_balance_response(),
                GetAssetsFilters {
                    asset_filter: None,
                    asset_type_filter: None,
                    chain_filter: Some(vec![U64::from(0x2105)]),
                },
            )
            .unwrap();

            assert_eq!(
                result[&U64::from(0x2105)]
                    .iter()
                    .find(|a| matches!(
                        a,
                        Asset::Erc20 {
                            data: AssetData {
                                metadata: Erc20Metadata { symbol, .. },
                                ..
                            }
                        } if symbol == "USDC"
                    ))
                    .unwrap()
                    .balance(),
                U64::from(0x26fdd0)
            );
        }

        // TODO
    }

    // TOOD
}
