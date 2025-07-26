use crate::{
    database::connection::DbPool,
    models::{
        auth::{AuthResponse, LoginRequest, RegisterRequest, UserInfo},
        user::{CreateUser, UserRole, User},
    },
    services::auth::AuthService,
    utils::helpers::ApiResponse,
};
use actix_web::{web, HttpResponse, Result};
use tracing::error;

pub async fn register(
    pool: web::Data<DbPool>,
    request: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    let auth_service = AuthService::new().map_err(|e| {
        error!("Failed to create auth service: {}", e);
        actix_web::error::ErrorInternalServerError("Authentication service error")
    })?;

    let user_role = match request.user_role.as_ref() {
        Some(role_str) => role_str.parse().unwrap_or_else(|_| {
            UserRole::Member
        }),
        None => UserRole::Member,
    };

    let create_user = CreateUser {
        fullname: request.fullname.clone(),
        email: request.email.clone(),
        password_hash: request.password.clone(),
        user_role,
    };

    let user = User::create(&pool, create_user).await.map_err(|e| {
        error!("Failed to create user: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to create user")
    })?;
    
    let token = auth_service.generate_token(&user).map_err(|e| {
        error!("Failed to generate token: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to generate token")
    })?;
    
    let user_info = UserInfo {
        id: user.id,
        fullname: user.fullname,
        email: user.email,
    };
    
    let response = AuthResponse {
        token,
        user: user_info,
    };
    
    Ok(HttpResponse::Created().json(ApiResponse::success(response)))
}

pub async fn login(
    pool: web::Data<DbPool>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let auth_service = AuthService::new().map_err(|e| {
        error!("Failed to create auth service: {}", e);
        actix_web::error::ErrorInternalServerError("Authentication service error")
    })?;

    let user = auth_service
        .authenticate_user(&pool, &request.email, &request.password)
        .await
        .map_err(|e| {
            error!("Authentication error: {}", e);
            actix_web::error::ErrorInternalServerError("Authentication error")
        })?
        .ok_or_else(|| {
            error!("Invalid credentials for user: {}", request.email);
            actix_web::error::ErrorUnauthorized("Invalid credentials")
        })?;

    let token = auth_service.generate_token(&user).map_err(|e| {
        error!("Failed to generate token: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to generate token")
    })?;

    let user_info = UserInfo {
        id: user.id,
        fullname: user.fullname,
        email: user.email,
    };

    let response = AuthResponse {
        token,
        user: user_info,
    };
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}
