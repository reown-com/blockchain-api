use {
    super::{super::SdkInfoParams, RegisterPayload, RegisterRequest, UNIXTIMESTAMP_SYNC_THRESHOLD},
    crate::{
        analytics::{AccountNameRegistration, MessageSource},
        database::{
            helpers::{get_name_and_addresses_by_name, insert_name},
            types::{Address, ENSIP11AddressesMap, SupportedNamespaces},
        },
        error::RpcError,
        names::{
            utils::{
                check_attributes, is_name_format_correct, is_name_in_allowed_zones,
                is_name_length_correct, is_timestamp_within_interval,
            },
            ATTRIBUTES_VALUE_MAX_LENGTH, SUPPORTED_ATTRIBUTES,
        },
        state::AppState,
        utils::{
            crypto::{
                convert_coin_type_to_evm_chain_id, is_coin_type_supported, verify_message_signature,
            },
            network,
            simple_request_json::SimpleRequestJson,
        },
    },
    axum::{
        extract::{ConnectInfo, Query, State},
        response::{IntoResponse, Response},
        Json,
    },
    hyper::{HeaderMap, StatusCode},
    serde::Deserialize,
    sqlx::Error as SqlxError,
    std::{collections::HashMap, net::SocketAddr, sync::Arc},
    tracing::log::error,
    wc::metrics::{future_metrics, FutureExt},
};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegisterQueryParams {
    #[serde(flatten)]
    pub sdk_info: SdkInfoParams,
}

pub async fn handler(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<RegisterQueryParams>,
    SimpleRequestJson(register_request): SimpleRequestJson<RegisterRequest>,
) -> Result<Response, RpcError> {
    handler_internal(state, connect_info, headers, query, register_request)
        .with_metrics(future_metrics!("handler:profile_register"))
        .await
}

#[tracing::instrument(skip(state), level = "debug")]
pub async fn handler_internal(
    state: State<Arc<AppState>>,
    connect_info: ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    query: Query<RegisterQueryParams>,
    register_request: RegisterRequest,
) -> Result<Response, RpcError> {
    let allowed_zones = state.config.names.allowed_zones.as_ref().ok_or_else(|| {
        RpcError::InvalidConfiguration("Names allowed zones are not defined".to_string())
    })?;
    let raw_payload = &register_request.message;
    let payload = match serde_json::from_str::<RegisterPayload>(raw_payload) {
        Ok(payload) => payload,
        Err(e) => return Err(RpcError::SerdeJson(e)),
    };

    // Check if the name is in the correct format
    if !is_name_format_correct(&payload.name) {
        return Err(RpcError::InvalidNameFormat(payload.name));
    }

    // Check if the name length is correct
    if !is_name_length_correct(&payload.name) {
        return Err(RpcError::InvalidNameLength(payload.name));
    }

    // Allow register only in the main zone
    if !is_name_in_allowed_zones(&payload.name, allowed_zones.clone()) {
        return Err(RpcError::InvalidNameZone(payload.name));
    }

    // Check for the supported ENSIP-11 coin type
    if !is_coin_type_supported(register_request.coin_type) {
        return Err(RpcError::UnsupportedCoinType(register_request.coin_type));
    }

    // Check is name already registered
    if get_name_and_addresses_by_name(payload.name.clone(), &state.postgres.clone())
        .await
        .is_ok()
    {
        return Err(RpcError::NameAlreadyRegistered(payload.name.clone()));
    };

    // Check the timestamp is within the sync threshold interval
    if !is_timestamp_within_interval(payload.timestamp, UNIXTIMESTAMP_SYNC_THRESHOLD) {
        return Err(RpcError::ExpiredTimestamp(payload.timestamp));
    }

    // Check for supported attributes
    if let Some(attributes) = payload.attributes.clone() {
        if !check_attributes(
            &attributes,
            &SUPPORTED_ATTRIBUTES,
            ATTRIBUTES_VALUE_MAX_LENGTH,
        ) {
            return Err(RpcError::UnsupportedNameAttribute);
        }
    }

    // Check the signature
    let chain_id_caip2 = format!(
        "eip155:{}",
        convert_coin_type_to_evm_chain_id(register_request.coin_type) as u64
    );
    let rpc_project_id = state
        .config
        .server
        .testing_project_id
        .as_ref()
        .ok_or_else(|| {
            RpcError::InvalidConfiguration(
                "Missing testing project id in the configuration for eip1271 lookups".to_string(),
            )
        })?;
    let sinature_check = match verify_message_signature(
        raw_payload,
        &register_request.signature,
        &register_request.address,
        &chain_id_caip2,
        rpc_project_id,
        MessageSource::ProfileRegisterSigValidate,
        None,
    )
    .await
    {
        Ok(sinature_check) => sinature_check,
        Err(_) => {
            return Err(RpcError::SignatureValidationError(
                "Invalid signature".into(),
            ))
        }
    };
    if !sinature_check {
        return Err(RpcError::SignatureValidationError(
            "Signature verification error".into(),
        ));
    }

    // Register (insert) a new domain with address
    let mut addresses: ENSIP11AddressesMap = HashMap::from([(
        register_request.coin_type,
        Address {
            address: register_request.address.clone(),
            created_at: None,
        },
    )]);

    // Adding address with cointype 60 (Mainnet) by default
    // if it was not provided during the registration
    if let std::collections::hash_map::Entry::Vacant(e) = addresses.entry(60) {
        e.insert(Address {
            address: register_request.address.clone(),
            created_at: None,
        });
    }

    let insert_result = insert_name(
        payload.name.clone(),
        payload.attributes.unwrap_or(HashMap::new()),
        SupportedNamespaces::Eip155,
        addresses,
        &state.postgres,
    )
    .await;
    if let Err(e) = insert_result {
        error!("Failed to insert new name: {e}");
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
    }

    // Name registration analytics
    {
        let origin = headers
            .get("origin")
            .map(|v| v.to_str().unwrap_or("invalid_header").to_string());
        let (country, continent, region) = state
            .analytics
            .lookup_geo_data(
                network::get_forwarded_ip(&headers).unwrap_or_else(|| connect_info.0.ip()),
            )
            .map(|geo| (geo.country, geo.continent, geo.region))
            .unwrap_or((None, None, None));
        state
            .analytics
            .name_registration(AccountNameRegistration::new(
                payload.name.clone(),
                register_request.address.clone(),
                chain_id_caip2,
                origin,
                region,
                country,
                continent,
                query.sdk_info.sv.clone(),
                query.sdk_info.st.clone(),
            ));
    }

    // Return the registered name and addresses
    match get_name_and_addresses_by_name(payload.name.clone(), &state.postgres.clone()).await {
        Ok(response) => Ok(Json(response).into_response()),
        Err(e) => match e {
            SqlxError::RowNotFound => Err(RpcError::NameRegistrationError(
                "Name was not found in the database after the registration".into(),
            )),
            _ => {
                // Handle other types of errors
                error!("Failed to lookup name: {e}");
                return Ok((StatusCode::INTERNAL_SERVER_ERROR, "").into_response());
            }
        },
    }
}
