use {
    crate::database::{error::DatabaseError, types, utils},
    sqlx::{PgPool, Postgres},
    std::collections::HashMap,
    tracing::instrument,
};

/// Initial name registration insert
#[instrument(skip(postgres))]
pub async fn insert_name(
    name: String,
    attributes: HashMap<String, String>,
    addresses: Vec<types::Address>,
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
        insert_address(
            name.clone(),
            address.namespace,
            address.chain_id,
            address.address,
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
pub async fn update_name(
    name: String,
    attributes: HashMap<String, String>,
    postgres: &PgPool,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::error::Error> {
    let insert_name_query = "
      UPDATE names SET attributes = $2::hstore, updated_at = NOW()
        WHERE name = $1
    ";
    sqlx::query::<Postgres>(insert_name_query)
        .bind(&name)
        // Convert JSON to String for hstore update
        .bind(&utils::hashmap_to_hstore(&attributes))
        .execute(postgres)
        .await
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
) -> Result<Vec<types::Address>, sqlx::error::Error> {
    let query = "
      SELECT namespace, chain_id, address, created_at
      FROM addresses
        WHERE name = $1
    ";
    sqlx::query_as::<Postgres, types::Address>(query)
        .bind(name)
        .fetch_all(postgres)
        .await
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
    chain_id: Option<String>,
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
    .bind(chain_id.unwrap_or_default())
    .bind(&address);

    query
        .execute(postgres)
        .await
        .map_err(DatabaseError::SqlxError)
}

#[instrument(skip(postgres))]
pub async fn insert_address<'e>(
    name: String,
    namespace: types::SupportedNamespaces,
    chain_id: Option<String>,
    address: String,
    postgres: impl sqlx::PgExecutor<'e>,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::error::Error> {
    let query = sqlx::query::<Postgres>(
        "
        INSERT INTO addresses (name, namespace, chain_id, address)
        VALUES ($1, $2, $3, $4)
        ",
    )
    .bind(&name)
    .bind(&namespace)
    .bind(chain_id.unwrap_or_default())
    .bind(&address);

    query.execute(postgres).await
}
