// use crate::services::auth::AuthService;
use actix::Actor;
use actix_cors::Cors;
use actix_web::{http, middleware::Logger, web, App, HttpServer};
use config::settings::AppConfig;
use database::connection::{create_pool, run_migrations};
use dotenv::dotenv;
use tracing::{error, info};
use tracing_subscriber;

mod config;
mod database;
mod handlers;
mod models;
mod requests;
mod routes;
mod services;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = AppConfig::from_env().unwrap_or_else(|e| {
        error!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    info!("Starting server with config: {:?}", config);

    let pool = create_pool(&config.database).await.unwrap_or_else(|e| {
        error!("Failed to create database pool: {}", e);
        std::process::exit(1);
    });

    info!("Database pool created successfully");

    run_migrations(&pool).await.unwrap_or_else(|e| {
        error!("Failed to run database migrations: {}", e);
        std::process::exit(1);
    });

    info!("Database migrations completed successfully");

    let server_host = config.server.host.clone();
    let server_port = config.server.port;
    
    // let auth_service = AuthService::new().unwrap_or_else(|e| {
    //     error!("Failed to create auth service: {}", e);
    //     std::process::exit(1);
    // });
    // let auth_service = std::sync::Arc::new(auth_service);

    //use: http://localhost:8080/api/v1/users to test
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().starts_with(b"http://localhost")
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            // .app_data(web::Data::from(auth_service.clone()))
            .wrap(Logger::default())
            .service(web::scope("/api/v1").configure(routes::api::scoped_config))
    })
        .bind((server_host, server_port))?
        .run()
        .await?;

    Ok(())
}

