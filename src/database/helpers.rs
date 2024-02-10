use {
    crate::database::{error::DatabaseError, types, utils},
    chrono::{DateTime, Utc},
    sqlx::{PgPool, Postgres, Row},
    std::collections::HashMap,
    tracing::{error, instrument},
};

#[derive(sqlx::FromRow)]
struct RowAddress {
    namespace: types::SupportedNamespaces,
    chain_id: String,
    address: String,
    created_at: DateTime<Utc>,
}

/// Initial name registration insert
#[instrument(skip(postgres))]
pub async fn insert_name(
    name: String,
    attributes: HashMap<String, String>,
    namespace: types::SupportedNamespaces,
    addresses: types::ENSIP11AddressesMap,
    postgres: &PgPool,
) -> Result<(), DatabaseError> {
    if addresses.is_empty() {
        return Err(DatabaseError::BadArgument(
            "At least one address is required for the new name".to_string(),
        ));
    }
    let mut transaction = postgres.begin().await?;
    let insert_name_query = "
      INSERT INTO names (name, attributes)
        VALUES ($1, $2::hstore)
    ";
    sqlx::query::<Postgres>(insert_name_query)
        .bind(&name.clone())
        // Convert JSON to String for hstore update
        .bind(&utils::hashmap_to_hstore(&attributes))
        .execute(&mut *transaction)
        .await?;

    for address in addresses {
        insert_or_update_address(
            name.clone(),
            namespace.clone(),
            format!("{}", address.0),
            address.1.address,
            &mut *transaction,
        )
        .await?;
    }
    transaction.commit().await.map_err(DatabaseError::SqlxError)
}

#[instrument(skip(postgres))]
pub async fn delete_name(
    name: String,
    postgres: &PgPool,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::error::Error> {
    let query = "
      DELETE FROM names WHERE name = $1
    ";
    sqlx::query::<Postgres>(query)
        .bind(name)
        .execute(postgres)
        .await
}

#[instrument(skip(postgres))]
pub async fn update_name_attributes(
    name: String,
    attributes: HashMap<String, String>,
    postgres: &PgPool,
) -> Result<HashMap<String, String>, DatabaseError> {
    let update_attributes_query = "
      UPDATE names SET attributes = $2::hstore, updated_at = NOW()
        WHERE name = $1 
        RETURNING attributes::json
    ";
    let row = sqlx::query(update_attributes_query)
        .bind(&name)
        .bind(&utils::hashmap_to_hstore(&attributes))
        .fetch_one(postgres)
        .await?;
    let result: serde_json::Value = row.get(0);
    let updated_attributes_result: Result<HashMap<String, String>, DatabaseError> =
        serde_json::from_value(result.clone()).map_err(|e| {
            error!("Failed to deserialize updated attributes: {}", e);
            DatabaseError::SerdeJson(e)
        });

    updated_attributes_result
}

#[instrument(skip(postgres))]
pub async fn get_name(name: String, postgres: &PgPool) -> Result<types::Name, sqlx::error::Error> {
    let query = "
      SELECT name, registered_at, updated_at, hstore_to_json(attributes) AS attributes
        FROM names
          WHERE name = $1
    ";
    sqlx::query_as::<Postgres, types::Name>(query)
        .bind(name)
        .fetch_one(postgres)
        .await
}

#[instrument(skip(postgres))]
pub async fn get_names_by_address(
    address: String,
    postgres: &PgPool,
) -> Result<Vec<types::Name>, sqlx::error::Error> {
    let query = "
        SELECT
            n.name,
            n.registered_at,
            n.updated_at,
            hstore_to_json(n.attributes) AS attributes
        FROM
            names n
        INNER JOIN
            addresses a ON n.name = a.name
        WHERE
            a.address = $1
    ";
    sqlx::query_as::<Postgres, types::Name>(query)
        .bind(address)
        .fetch_all(postgres)
        .await
}

#[instrument(skip(postgres))]
pub async fn get_addresses_by_name(
    name: String,
    postgres: &PgPool,
) -> Result<types::ENSIP11AddressesMap, sqlx::error::Error> {
    let query = "
      SELECT namespace, chain_id, address, created_at
      FROM addresses
        WHERE name = $1
    ";

    let rows_result = sqlx::query_as::<Postgres, RowAddress>(query)
        .bind(name)
        .fetch_all(postgres)
        .await?;

    let mut result_map = types::ENSIP11AddressesMap::new();

    for row in rows_result {
        if row.namespace != types::SupportedNamespaces::Eip155 {
            error!("Unsupported namespace: {:?}", row.namespace);
            continue;
        }

        result_map.insert(
            row.chain_id.parse::<u32>().unwrap_or_default(),
            types::Address {
                address: row.address,
                created_at: Some(row.created_at),
            },
        );
    }

    Ok(result_map)
}

#[instrument(skip(postgres))]
pub async fn get_names_by_address_and_namespace(
    address: String,
    namespace: types::SupportedNamespaces,
    postgres: &PgPool,
) -> Result<Vec<types::Name>, sqlx::error::Error> {
    let query = "
        SELECT 
            n.name, 
            n.registered_at, 
            n.updated_at, 
            hstore_to_json(n.attributes) AS attributes
        FROM 
            names n
        INNER JOIN 
            addresses a ON n.name = a.name
        WHERE 
            a.address = $1 AND a.namespace = $2
    ";
    sqlx::query_as::<Postgres, types::Name>(query)
        .bind(address)
        .bind(namespace)
        .fetch_all(postgres)
        .await
}

#[instrument(skip(postgres))]
pub async fn get_name_and_addresses_by_name(
    name: String,
    postgres: &PgPool,
) -> Result<types::NameAndAddresses, sqlx::error::Error> {
    let result = get_name(name.clone(), postgres).await?;
    let addresses = get_addresses_by_name(name, postgres).await?;

    Ok(types::NameAndAddresses {
        name: result.name,
        registered_at: result.registered_at,
        updated_at: result.updated_at,
        attributes: result.attributes,
        addresses,
    })
}

#[instrument(skip(postgres))]
pub async fn delete_address(
    name: String,
    namespace: types::SupportedNamespaces,
    chain_id: String,
    address: String,
    postgres: &PgPool,
) -> Result<sqlx::postgres::PgQueryResult, DatabaseError> {
    let current_addresses = get_addresses_by_name(name.clone(), postgres).await?;
    if current_addresses.len() == 1 {
        return Err(DatabaseError::AddressRequired(
            "At least one address is required to exist for the name".to_string(),
        ));
    }
    let query = sqlx::query::<Postgres>(
        "
        DELETE FROM addresses
        WHERE
            name = $1 AND
            namespace = $2 AND
            chain_id = $3 AND
            address = $4
        ",
    )
    .bind(&name)
    .bind(&namespace)
    .bind(chain_id)
    .bind(&address);

    query
        .execute(postgres)
        .await
        .map_err(DatabaseError::SqlxError)
}

#[instrument(skip(postgres))]
pub async fn insert_or_update_address<'e>(
    name: String,
    namespace: types::SupportedNamespaces,
    chain_id: String,
    address: String,
    postgres: impl sqlx::PgExecutor<'e>,
) -> Result<types::ENSIP11AddressesMap, sqlx::error::Error> {
    let insert_or_update_query = "
        INSERT INTO addresses (name, namespace, chain_id, address)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (name, namespace, chain_id, address) DO UPDATE
        SET address = EXCLUDED.address, created_at = NOW()
        RETURNING *
        ";
    let row_result = sqlx::query_as::<Postgres, RowAddress>(insert_or_update_query)
        .bind(&name)
        .bind(&namespace)
        .bind(chain_id)
        .bind(&address)
        .fetch_one(postgres)
        .await?;

    let mut result_map = types::ENSIP11AddressesMap::new();
    result_map.insert(
        row_result.chain_id.parse::<u32>().unwrap_or_default(),
        types::Address {
            address: row_result.address,
            created_at: Some(row_result.created_at),
        },
    );

    Ok(result_map)
}
