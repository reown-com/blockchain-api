use {
    crate::utils::{generate_random_string, get_postgres_pool},
    rpc_proxy::database::{
        helpers::{
            delete_address,
            delete_name,
            get_addresses_by_name,
            get_name,
            get_name_and_addresses_by_name,
            get_names_by_address,
            get_names_by_address_and_namespace,
            insert_name,
            insert_or_update_address,
            update_name_attributes,
        },
        types,
    },
    std::collections::HashMap,
};

#[tokio::test]
async fn insert_and_get_name_by_name() {
    let pg_pool = get_postgres_pool().await;

    let name = format!("{}.connect.id", generate_random_string(10));
    let address = format!("0x{}", generate_random_string(16));
    let chain_id = 1;
    let addresses = HashMap::from([(chain_id, types::Address {
        address,
        created_at: None,
    })]);

    // create a new hashmap with attributes
    let attributes: HashMap<String, String> = HashMap::from_iter([
        (
            "avatar".to_string(),
            "http://test.url/avatar.png".to_string(),
        ),
        ("bio".to_string(), "just about myself".to_string()),
    ]);

    let insert_result = insert_name(
        name.clone(),
        attributes.clone(),
        types::SupportedNamespaces::Eip155,
        addresses,
        &pg_pool,
    )
    .await;
    if let Err(ref e) = insert_result {
        println!("Error: {:?}", e);
    }
    assert!(insert_result.is_ok(), "Inserting a new name should succeed");

    let get_name_result = get_name(name.clone(), &pg_pool).await;
    assert!(
        get_name_result.is_ok(),
        "Getting name after inserting should succeed"
    );

    let got_name = get_name_result.unwrap();
    let got_attributes = got_name.attributes.unwrap();

    assert_eq!(got_name.name, name);
    assert_eq!(got_attributes["avatar"], attributes["avatar"]);
    assert_eq!(got_attributes["bio"], attributes["bio"]);

    // Cleanup
    let delete_result = delete_name(name, &pg_pool).await;
    assert!(delete_result.is_ok(), "Deleting name should succeed");
}

#[tokio::test]
async fn insert_and_get_names_by_address() {
    let pg_pool = get_postgres_pool().await;

    let name = format!("{}.connect.id", generate_random_string(10));
    let address = format!("0x{}", generate_random_string(16));
    let chain_id = 1;
    let addresses = HashMap::from([(chain_id, types::Address {
        address: address.clone(),
        created_at: None,
    })]);

    let insert_result = insert_name(
        name.clone(),
        HashMap::new(),
        types::SupportedNamespaces::Eip155,
        addresses,
        &pg_pool,
    )
    .await;
    assert!(insert_result.is_ok(), "Inserting a new name should succeed");

    let get_names_result = get_names_by_address(address, &pg_pool).await;
    assert!(
        get_names_result.is_ok(),
        "Getting name by the address after inserting should succeed"
    );

    let got_names = get_names_result.unwrap();
    assert_eq!(got_names[0].name, name);

    // Cleanup
    let delete_result = delete_name(name, &pg_pool).await;
    assert!(delete_result.is_ok(), "Deleting name should succeed");
}

#[tokio::test]
async fn insert_and_get_names_by_address_and_namespace() {
    let pg_pool = get_postgres_pool().await;

    let name = format!("{}.connect.id", generate_random_string(10));
    let address = format!("0x{}", generate_random_string(16));
    let namespace = types::SupportedNamespaces::Eip155;
    let chain_id = 1;
    let addresses = HashMap::from([(chain_id, types::Address {
        address: address.clone(),
        created_at: None,
    })]);

    let insert_result = insert_name(
        name.clone(),
        HashMap::new(),
        types::SupportedNamespaces::Eip155,
        addresses,
        &pg_pool,
    )
    .await;
    assert!(insert_result.is_ok(), "Inserting a new name should succeed");

    let get_names_result = get_names_by_address_and_namespace(address, namespace, &pg_pool).await;
    assert!(
        get_names_result.is_ok(),
        "Getting name by the address after inserting should succeed"
    );

    let got_names = get_names_result.unwrap();
    assert_eq!(got_names[0].name, name);

    // Cleanup
    let delete_result = delete_name(name, &pg_pool).await;
    assert!(delete_result.is_ok(), "Deleting name should succeed");
}

#[tokio::test]
async fn insert_and_get_name_and_addresses() {
    let pg_pool = get_postgres_pool().await;

    let name = format!("{}.connect.id", generate_random_string(10));
    let address = format!("0x{}", generate_random_string(16));
    let namespace = types::SupportedNamespaces::Eip155;
    let expected_ensip11_coin_type = 60;
    let addresses = HashMap::from([(expected_ensip11_coin_type, types::Address {
        address: address.clone(),
        created_at: None,
    })]);

    let attributes: HashMap<String, String> = HashMap::from_iter([(
        "avatar".to_string(),
        "http://test.url/avatar.png".to_string(),
    )]);

    let insert_result = insert_name(
        name.clone(),
        attributes.clone(),
        namespace,
        addresses,
        &pg_pool,
    )
    .await;
    assert!(insert_result.is_ok(), "Inserting a new name should succeed");

    let get_name_result = get_name_and_addresses_by_name(name.clone(), &pg_pool).await;
    assert!(
        get_name_result.is_ok(),
        "Getting name after inserting should succeed"
    );

    let got_name = get_name_result.unwrap();
    assert_eq!(got_name.name, name);
    assert_eq!(got_name.attributes.unwrap()["avatar"], attributes["avatar"]);
    assert_eq!(
        got_name
            .addresses
            .get(&expected_ensip11_coin_type)
            .unwrap()
            .address,
        address
    );

    // Cleanup
    let delete_result = delete_name(name, &pg_pool).await;
    assert!(delete_result.is_ok(), "Deleting name should succeed");
}

#[tokio::test]
async fn insert_and_update_name_attributes() {
    let pg_pool = get_postgres_pool().await;

    let name = format!("{}.connect.id", generate_random_string(10));
    let address = format!("0x{}", generate_random_string(16));
    let chain_id = 1;
    let addresses = HashMap::from([(chain_id, types::Address {
        address,
        created_at: None,
    })]);

    // create a new hashmap with attributes
    let attributes: HashMap<String, String> = HashMap::from_iter([
        (
            "avatar".to_string(),
            "http://test.url/avatar.png".to_string(),
        ),
        ("bio".to_string(), "just about myself".to_string()),
    ]);

    let insert_result = insert_name(
        name.clone(),
        attributes.clone(),
        types::SupportedNamespaces::Eip155,
        addresses,
        &pg_pool,
    )
    .await;
    assert!(insert_result.is_ok(), "Inserting a new name should succeed");

    let get_name_result = get_name(name.clone(), &pg_pool).await;
    assert!(
        get_name_result.is_ok(),
        "Getting name after inserting should succeed"
    );

    let got_name = get_name_result.unwrap();
    let got_attributes = got_name.attributes.unwrap();

    assert_eq!(got_name.name, name.clone());
    assert_eq!(got_attributes["avatar"], attributes["avatar"]);
    assert_eq!(got_attributes["bio"], attributes["bio"]);

    // Updating the name with new attributes
    let updated_attributes: HashMap<String, String> =
        HashMap::from_iter([("GitHub".to_string(), "SomeProfile".to_string())]);
    let updated_result =
        update_name_attributes(name.clone(), updated_attributes.clone(), &pg_pool).await;
    assert!(updated_result.is_ok(), "Updating name should succeed");

    let got_update_name = get_name(name.clone(), &pg_pool).await.unwrap();
    assert_eq!(got_update_name.name, name.clone());
    assert_eq!(
        got_update_name.attributes.unwrap()["GitHub"],
        updated_attributes["GitHub"]
    );

    // Cleanup
    let delete_result = delete_name(name, &pg_pool).await;
    assert!(delete_result.is_ok(), "Deleting name should succeed");
}

#[tokio::test]
async fn insert_delete_two_addresses() {
    let pg_pool = get_postgres_pool().await;

    let name = format!("{}.connect.id", generate_random_string(10));
    let address = format!("0x{}", generate_random_string(16));
    let mut chain_id = 1;
    let addresses = HashMap::from([(chain_id, types::Address {
        address: address.clone(),
        created_at: None,
    })]);

    let insert_result = insert_name(
        name.clone(),
        HashMap::new(),
        types::SupportedNamespaces::Eip155,
        addresses,
        &pg_pool,
    )
    .await;
    assert!(insert_result.is_ok(), "Inserting a new name should succeed");

    let delete_address_result = delete_address(
        name.clone(),
        types::SupportedNamespaces::Eip155,
        format!("{}", chain_id),
        address.clone(),
        &pg_pool,
    )
    .await;
    // At least one address is required to exist for the name
    assert!(delete_address_result.is_err());

    // Inserting a new address
    chain_id = 137;
    let new_address = format!("0x{}", generate_random_string(16));
    let insert_address_result = insert_or_update_address(
        name.clone(),
        types::SupportedNamespaces::Eip155,
        format!("{}", chain_id),
        new_address.clone(),
        &pg_pool,
    )
    .await;
    assert!(insert_address_result.is_ok());

    // Check for two addresses for the name
    let current_addresses = get_addresses_by_name(name.clone(), &pg_pool).await;
    assert!(insert_address_result.is_ok());
    let current_addresses = current_addresses.unwrap();
    assert_eq!(current_addresses.len(), 2);

    // Deleting the address should succeed because there is more than one address
    let delete_address_result = delete_address(
        name.clone(),
        types::SupportedNamespaces::Eip155,
        format!("{}", chain_id),
        address,
        &pg_pool,
    )
    .await;
    assert!(delete_address_result.is_ok());

    // Cleanup
    let delete_result = delete_name(name, &pg_pool).await;
    assert!(delete_result.is_ok(), "Deleting name should succeed");
}
