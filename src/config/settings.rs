use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub environment: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let app_environment =
            env::var("APP_ENV").map_err(|e| ConfigError::NotFound(format!("APP_ENV: {}", e)))?;

        let database_url_key = match app_environment.as_str() {
            "development" => "LOCAL_DATABASE_URL",
            "production" => "PROD_DATABASE_URL",
            env => {
                return Err(ConfigError::Message(format!(
                    "Unsupported APP_ENV value '{}'. Valid values: development, production",
                    env
                )));
            }
        };

        let database_url = env::var(database_url_key)
            .map_err(|e| ConfigError::NotFound(format!("{}: {}", database_url_key, e)))?;

        let cfg = Config::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8080)?
            .set_default("database.max_connections", 20)?
            .set_default("database.min_connections", 5)?
            .set_default("database.connection_timeout", 30)? // seconds
            .set_default("database.idle_timeout", 600)? // seconds
            .add_source(Environment::with_prefix("APP").separator("__"))
            .set_override("database.url", database_url)?
            .set_override("environment", app_environment)?;

        // Build and deserialize
        let config: AppConfig = cfg.build()?.try_deserialize()?;

        // Validate database connections
        if config.database.min_connections > config.database.max_connections {
            return Err(ConfigError::Message(
                "min_connections cannot be greater than max_connections".to_string(),
            ));
        }

        Ok(config)
    }
}
