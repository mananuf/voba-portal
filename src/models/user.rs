use crate::database::connection::DbPool;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::str::FromStr;
use rand::distributions::Alphanumeric;
use rand::Rng;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User with ID {id} not found")]
    NotFound { id: Uuid },
    #[error("User with email {email} not found")]
    NotFoundByEmail { email: String },
    #[error("Email already exists: {email}")]
    EmailAlreadyExists { email: String },
    #[error("Invalid verification code")]
    InvalidVerificationCode,
    #[error("Verification code expired")]
    VerificationCodeExpired,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Password hashing error")]
    PasswordHash,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "user_roles", rename_all = "lowercase")]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Member,
    Treasurer,
}

impl FromStr for UserRole {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "super_admin" => Ok(UserRole::SuperAdmin),
            "admin" => Ok(UserRole::Admin),
            "member" => Ok(UserRole::Member),
            "treasurer" => Ok(UserRole::Treasurer),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub fullname: String,
    pub email: String,
    pub password_hash: String,
    pub phone: Option<String>,
    pub dob: Option<DateTime<Utc>>,
    pub photo_url: Option<String>,
    pub user_role: UserRole,
    pub email_verification_code: Option<String>,
    pub email_verification_expires_at: Option<DateTime<Utc>>,
    pub is_email_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub password_hash: String,
    pub user_role: UserRole,
    pub is_active: bool,
    // pub phone: Option<String>,
    // pub dob: Option<DateTime<Utc>>,
    // pub photo_url: Option<String>,
}

impl User {
    fn generate_verification_code() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    pub async fn create(pool: &DbPool, user: CreateUser) -> Result<Self, UserError> {
        if let Ok(Some(_)) = Self::find_by_email(pool, &user.email).await {
            return Err(UserError::EmailAlreadyExists { email: user.email });
        }
        
        let now = Utc::now();
        let hashed_password = hash(user.password_hash.as_bytes(), DEFAULT_COST)
            .map_err(|_| UserError::PasswordHash)?;

        let email_verification_code = Self::generate_verification_code();
        let email_verification_expires_at = now + Duration::hours(24);

        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (id, fullname, email, password_hash, user_role, email_verification_code, email_verification_expires_at, is_email_verified, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING *",
        )
            .bind(Uuid::new_v4())
            .bind(user.fullname)
            .bind(user.email)
            .bind(hashed_password)
            .bind(user.user_role)
            .bind(email_verification_code)
            .bind(email_verification_expires_at)
            .bind(false)
            .bind(user.is_active)
            .bind(now)
            .bind(now)
            .fetch_one(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, UserError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<Self>, UserError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_verification_code(pool: &DbPool, code: &str) -> Result<Option<Self>, UserError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, fullname, email, password_hash, phone, dob, photo_url, 
                   user_role, email_verification_code, 
                   email_verification_expires_at, is_email_verified, is_active, 
                   created_at, updated_at
            FROM users WHERE email_verification_code = $1
            "#,)
            .bind(code)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn verify_email(pool: &DbPool, verification_code: &str) -> Result<Self, UserError> {
        let user = Self::find_by_verification_code(pool, verification_code)
            .await?
            .ok_or(UserError::InvalidVerificationCode)?;
        
        if let Some(expires_at) = user.email_verification_expires_at {
            if Utc::now() > expires_at {
                return Err(UserError::VerificationCodeExpired);
            }
        }

        // Update user to mark email as verified
        let updated_user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET is_email_verified = true, 
                email_verification_code = NULL, 
                email_verification_expires_at = NULL,
                updated_at = $2
            WHERE id = $1 
            RETURNING id, fullname, email, password_hash, phone, dob, photo_url, 
                      user_role, email_verification_code, 
                      email_verification_expires_at, is_email_verified, is_active, 
                      created_at, updated_at
            "#,)
            .bind(user.id)
            .bind(Utc::now())
            .fetch_one(pool)
            .await?;

        Ok(updated_user)
    }

    pub async fn resend_verification_code(pool: &DbPool, email: &str) -> Result<Self, UserError> {
        let user = Self::find_by_email(pool, email)
            .await?
            .ok_or(UserError::NotFoundByEmail { email: email.to_string() })?;

        if user.is_email_verified {
            return Ok(user); // Already verified, no need to resend
        }

        let new_verification_code = Self::generate_verification_code();
        let new_expires_at = Utc::now() + Duration::hours(24);

    let updated_user = sqlx::query_as::<_, User >(
            r#"
            UPDATE users 
            SET email_verification_code = $2, 
                email_verification_expires_at = $3,
                updated_at = $4
            WHERE id = $1 
            RETURNING id, fullname, email, password_hash, phone, dob, photo_url, 
                      user_role as "user_role: UserRole", email_verification_code, 
                      email_verification_expires_at, is_email_verified, is_active, 
                      created_at, updated_at
            "#,)
            .bind(user.id)
            .bind(new_verification_code)
            .bind(new_expires_at)
            .bind(Utc::now())
            .fetch_one(pool)
            .await?;

        Ok(updated_user)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Self>, UserError> {
        let users = sqlx::query_as::<_, User>(
            r#"
                SELECT id, fullname, email, password_hash, phone, dob, photo_url, 
                       user_role, email_verification_code, 
                       email_verification_expires_at, is_email_verified, is_active, 
                       created_at, updated_at
                FROM users ORDER BY created_at DESC
                "#
            )
            .fetch_all(pool)
            .await?;

        Ok(users)
    }


    pub fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        verify(password, &self.password_hash)
    }

    pub async fn authenticate(
        pool: &DbPool,
        email: &str,
        password: &str,
    ) -> Result<Option<Self>, UserError> {
        if let Some(user) = Self::find_by_email(pool, email).await? {
            if user.verify_password(password).unwrap_or(false) {
                return Ok(Some(user));
            }
        }
        Ok(None)
    }

    pub async fn toggle_active(
        pool: &DbPool,
        user_id: Uuid
    ) -> Result<Self, UserError> {
        let user = Self::find_by_id(pool, user_id)
            .await?
            .ok_or(UserError::NotFound { id: user_id })?;

        let updated_user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET is_active = $2, 
                updated_at = $3
            WHERE id = $1 
            RETURNING *
            "#,
        )
            .bind(user.id)
            .bind(!user.is_active)
            .bind(Utc::now())
            .fetch_one(pool)
            .await?;

        Ok(updated_user)
    }
}
