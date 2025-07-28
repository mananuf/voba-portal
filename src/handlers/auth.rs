use crate::models::user::UserError;
use crate::requests::register::RegisterRequest;
use crate::requests::resend_email_verification::ResendVerificationRequest;
use crate::requests::verify_email::VerifyEmailRequest;
use crate::services::email::EmailService;
use crate::{
    database::connection::DbPool,
    models::{
        auth::{AuthResponse, LoginRequest, UserInfo},
        user::{CreateUser, User, UserRole},
    },
    services::auth::AuthService,
    utils::helpers::ApiResponse,
};
use actix_web::{HttpResponse, Result, web};
use tracing::error;
use tracing::log::info;

pub async fn register(
    pool: web::Data<DbPool>,
    request: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    let auth_service = AuthService::new().map_err(|e| {
        error!("Failed to create auth service: {}", e);
        actix_web::error::ErrorInternalServerError("Authentication service error")
    })?;

    let email_service = EmailService::new().map_err(|e| {
        error!("Failed to create email service: {}", e);
        actix_web::error::ErrorInternalServerError("Email service error")
    })?;

    let user_role = match request.user_role.as_ref() {
        Some(role_str) => role_str.parse().unwrap_or_else(|_| UserRole::Member),
        None => UserRole::Member,
    };

    let is_active = request.is_active.unwrap_or(false);

    let create_user = CreateUser {
        fullname: request.fullname.clone(),
        email: request.email.clone(),
        password_hash: request.password.clone(),
        user_role,
        is_active,
    };

    let user = match User::create(&pool, create_user).await {
        Ok(user) => user,
        Err(UserError::EmailAlreadyExists { email }) => {
            return Ok(
                HttpResponse::Conflict().json(ApiResponse::<()>::error(format!(
                    "Email {} already exists",
                    email
                ))),
            );
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to create user".to_string(),
                )),
            );
        }
    };

    if let Some(verification_code) = &user.email_verification_code {
        let template =
            email_service.generate_verification_template(&user.fullname, verification_code);

        if let Err(e) = email_service.send_email(&user.email, Some(&user.fullname), template) {
            error!("Failed to send verification email: {}", e);
            // Don't fail registration if email fails, but log it
        } else {
            info!("Verification email sent to: {}", user.email);
        }
    }

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

    Ok(
        HttpResponse::Created().json(ApiResponse::success_with_message(
            response,
            "Registration successful. Please check your email to verify your account.".to_string(),
        )),
    )
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

    if !user.is_active {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Account is not active".to_string(),
        )));
    }

    if !user.is_email_verified {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Please verify your email address before logging in".to_string(),
        )));
    }

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

pub async fn verify_email(
    pool: web::Data<DbPool>,
    query: web::Query<VerifyEmailRequest>,
) -> Result<HttpResponse> {
    let email_service = EmailService::new().map_err(|e| {
        error!("Failed to create email service: {}", e);
        actix_web::error::ErrorInternalServerError("Email service error")
    })?;

    let user = match User::verify_email(&pool, &query.code).await {
        Ok(user) => user,
        Err(UserError::InvalidVerificationCode) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Invalid verification code".to_string(),
            )));
        }
        Err(UserError::VerificationCodeExpired) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                "Verification code has expired".to_string(),
            )));
        }
        Err(e) => {
            error!("Email verification error: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Email verification failed".to_string(),
                )),
            );
        }
    };

    let welcome_template = email_service.generate_welcome_template(&user.fullname);
    if let Err(e) = email_service.send_email(&user.email, Some(&user.fullname), welcome_template) {
        error!("Failed to send welcome email: {}", e);
        // Don't fail verification if welcome email fails
    }

    info!("Email verified successfully for user: {}", user.email);

    Ok(
        HttpResponse::Ok().json(ApiResponse::<()>::success_with_message(
            (),
            "Email verified successfully! Welcome to Portal.".to_string(),
        )),
    )
}

pub async fn resend_verification(
    pool: web::Data<DbPool>,
    request: web::Json<ResendVerificationRequest>,
) -> Result<HttpResponse> {
    let email_service = EmailService::new().map_err(|e| {
        error!("Failed to create email service: {}", e);
        actix_web::error::ErrorInternalServerError("Email service error")
    })?;

    let user = match User::resend_verification_code(&pool, &request.email).await {
        Ok(user) => user,
        Err(UserError::NotFoundByEmail { email }) => {
            return Ok(
                HttpResponse::NotFound().json(ApiResponse::<()>::error(format!(
                    "User with email {} not found",
                    email
                ))),
            );
        }
        Err(e) => {
            error!("Failed to resend verification code: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to resend verification code".to_string(),
                )),
            );
        }
    };

    if user.is_email_verified {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "Email is already verified".to_string(),
        )));
    }

    if let Some(verification_code) = &user.email_verification_code {
        let template =
            email_service.generate_verification_template(&user.fullname, verification_code);

        if let Err(e) = email_service.send_email(&user.email, Some(&user.fullname), template) {
            error!("Failed to send verification email: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to send verification email".to_string(),
                )),
            );
        }
    }

    info!("Verification email resent to: {}", user.email);

    Ok(HttpResponse::Ok().json(ApiResponse::success_with_message(
        (),
        format!("Verification email resent to: {}", user.email),
    )))
}
