use petstore_hexarch_rust::inbound::http::{HttpServer, HttpServerConfig};
use petstore_hexarch_rust::domain::petstore::service::Service;
use petstore_hexarch_rust::outbound::connect::PostgresClient;
use petstore_hexarch_rust::outbound::params::ConnectionParams;

#[tokio::main]
async fn main() -> anyhow::Result<()> {


    // A minimal tracing middleware for request logging.
    tracing_subscriber::fmt::init();

    let params = ConnectionParams {
        host: "localhost".to_string(),
        port: 5432,
        dbname: "postgres".to_string(),
        user: "postgres".to_string(),
        password: "postgres".to_string(),
    };

    // Wait for database to be ready
    let mut retries = 5;
    let client = loop {
        match PostgresClient::new(&params).await {
            Ok(client) => break client,
            Err(e) if retries > 0 => {
                eprintln!("Failed to connect to database, retrying... ({})", e);
                retries -= 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    };

    sqlx::migrate!("./migrations")
        .run(client.pool())
        .await
        .expect("Failed to run migrations");
    
    let pet_service = Service::new(client);

    let server_config = HttpServerConfig {
        port: "8080",
    };
    let http_server = HttpServer::new(pet_service, server_config).await?;
    http_server.run().await
}
