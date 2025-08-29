use testcontainers::{core::{WaitFor, IntoContainerPort}, runners::AsyncRunner, GenericImage, ImageExt};
use sqlx::postgres::PgPoolOptions;
use tokio::time::{sleep, Duration};
use petstore_hexarch_rust::domain::petstore::models::pet::{CreatePetRequest, Status};
use petstore_hexarch_rust::domain::petstore::models::category::Category;
use petstore_hexarch_rust::domain::petstore::models::tag::Tag;
use petstore_hexarch_rust::outbound::connect::PostgresClient;
use petstore_hexarch_rust::outbound::params::ConnectionParams;
use petstore_hexarch_rust::domain::petstore::ports::PetRepository;


#[tokio::test]
async fn test_database_connection() {
    // Start a PostgreSQL container
    let container = GenericImage::new("postgres", "latest")
        .with_wait_for(WaitFor::message_on_stdout("database system is ready to accept connections"))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .expect("Failed to start container");

    // Get connection details
    let host_port = container.get_host_port_ipv4(5432)
        .await
        .expect("Failed to get host port");
    let conn_string = format!(
        "postgres://{user}:{password}@{host}:{port}/{db}",
        user = "postgres",
        password = "postgres",
        host = "localhost",
        port = host_port,
        db = "postgres"
    );

    // Give the database a moment to fully start up (optional, but good practice)
    sleep(Duration::from_secs(1)).await;

    // 4. Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conn_string)
        .await
        .expect("Failed to connect to PostgreSQL");

    // 5. Perform a test query
    let row: (i32,) = sqlx::query_as("SELECT 1 + 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to execute query");

    assert_eq!(row.0, 2);

    println!("Successfully connected to PostgreSQL and ran a query!");

    // Testcontainers will automatically terminate the container when `container` goes out of scope.
}

#[tokio::test]
async fn test_add_pet() {
    // Start a PostgreSQL container
    let container = GenericImage::new("postgres", "latest")
        .with_wait_for(WaitFor::message_on_stdout("database system is ready to accept connections"))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .expect("Failed to start container");

    // Get connection details
    let host_port = container.get_host_port_ipv4(5432)
        .await
        .expect("Failed to get host port");
    let conn_string = format!(
        "postgres://{user}:{password}@{host}:{port}/{db}",
        user = "postgres",
        password = "postgres",
        host = "localhost",
        port = host_port,
        db = "postgres"
    );

    sleep(Duration::from_secs(1)).await;

    // Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conn_string)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create PostgresClient
    let params = ConnectionParams {
        host: "localhost".to_string(),
        port: host_port,
        dbname: "postgres".to_string(),
        user: "postgres".to_string(),
        password: "postgres".to_string(),
    };
    let client = PostgresClient::new(&params).await.expect("Failed to create PostgresClient");

    // Test adding a pet with all fields
    let category = Category::with_values(1, "Dogs".to_string());
    let tags = vec![
        Tag::with_values(1, "friendly".to_string()),
        Tag::with_values(2, "playful".to_string()),
    ];
    let photo_urls = vec![
        "http://example.com/photo1.jpg".to_string(),
        "http://example.com/photo2.jpg".to_string(),
    ];

    let req = CreatePetRequest::new(
        Some(1),
        "Buddy".to_string(),
        Some(category),
        photo_urls,
        tags,
        Some(Status::Available),
    );

    // Add the pet
    let pet = client.add_pet(&req)
        .await
        .expect("Failed to add pet");

    // Verify the pet was added correctly
    assert!(pet.id.is_some());
    assert_eq!(pet.status, Some(Status::Available));
    assert_eq!(pet.photo_urls.len(), 2);
    assert_eq!(pet.tags.len(), 2);

    // Test duplicate pet name
    let duplicate_req = CreatePetRequest::new(
        Some(1),
        "Buddy".to_string(),
        None,
        vec![],
        vec![],
        None,
    );

    let result = client.add_pet(&duplicate_req).await;
    assert!(result.is_err());

    // Verify the database state
    let pet_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pets")
        .fetch_one(client.pool())
        .await
        .expect("Failed to query pet count");

    assert_eq!(pet_count, 1, "Should only have one pet in the database");

    let photo_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pet_photos")
        .fetch_one(client.pool())
        .await
        .expect("Failed to query photo count");

    assert_eq!(photo_count, 2, "Should have two photos");

    let tag_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
        .fetch_one(client.pool())
        .await
        .expect("Failed to query tag count");

    assert_eq!(tag_count, 2, "Should have two tags");

    let pet_tag_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pet_tags")
        .fetch_one(client.pool())
        .await
        .expect("Failed to query pet_tag count");

    assert_eq!(pet_tag_count, 2, "Should have two pet-tag relationships");
}

#[tokio::test]
async fn test_find_pet_by_id() {
    // Start a PostgreSQL container
    let container = GenericImage::new("postgres", "latest")
        .with_wait_for(WaitFor::message_on_stdout("database system is ready to accept connections"))
        .with_exposed_port(5432.tcp())
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .expect("Failed to start container");

    // Get connection details
    let host_port = container.get_host_port_ipv4(5432)
        .await
        .expect("Failed to get host port");
    let conn_string = format!(
        "postgres://{user}:{password}@{host}:{port}/{db}",
        user = "postgres",
        password = "postgres",
        host = "localhost",
        port = host_port,
        db = "postgres"
    );

    sleep(Duration::from_secs(1)).await;

    // Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&conn_string)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create PostgresClient
    let params = ConnectionParams {
        host: "localhost".to_string(),
        port: host_port,
        dbname: "postgres".to_string(),
        user: "postgres".to_string(),
        password: "postgres".to_string(),
    };
    let client = PostgresClient::new(&params).await.expect("Failed to create PostgresClient");

    // Test adding a pet with all fields
    let category = Category::with_values(1, "Dogs".to_string());
    let tags = vec![
        Tag::with_values(0, "string".to_string()),
    ];
    let photo_urls = vec!["string".to_string()];
    let status = Status::Available;

    let req = CreatePetRequest::new(
        Some(10),
        "doggie".to_string(),
        Some(category),
        photo_urls.clone(),
        tags.clone(),
        Some(status),
    );

    // Add the pet
    let pet = client.add_pet(&req)
        .await
        .expect("Failed to add pet");

    // Verify the pet was added correctly
    assert!(pet.id.is_some());
    assert_eq!(pet.name, "doggie");
    assert!(pet.category.is_some());
    assert_eq!(pet.photo_urls, photo_urls);
    assert_eq!(pet.tags, tags);
    assert_eq!(pet.status, Some(Status::Available));

    // Test finding the pet
    let found = client.find_pet_by_id(10)
        .await
        .expect("Failed to find pet");

    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, Some(10));
    assert_eq!(found.name, "doggie");
    assert!(found.category.is_some());
    let category = found.category.unwrap();
    assert_eq!(category.id, Some(1));
    assert_eq!(category.name, Some("Dogs".to_string()));
    assert_eq!(found.photo_urls, photo_urls);
    assert_eq!(found.tags, tags);
    assert_eq!(found.status, Some(Status::Available));

    // Test finding non-existent pet
    let not_found = client.find_pet_by_id(999)
        .await
        .expect("Failed to query non-existent pet");
    assert!(not_found.is_none());
}