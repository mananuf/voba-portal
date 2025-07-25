use crate::config::settings::DatabaseConfig;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;

pub type DbPool = Pool<Postgres>;

pub async fn create_pool(config: &DatabaseConfig) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connection_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .connect(&config.url)
        .await
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
