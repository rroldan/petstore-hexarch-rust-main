use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::outbound::params::ConnectionParams;

#[derive(Debug, Clone)]
pub struct PostgresClient {
    pool: PgPool,
}

impl PostgresClient {
    pub async fn new(params: &ConnectionParams) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(params.connect_string().as_str())
            .await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
