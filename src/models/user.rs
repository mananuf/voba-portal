use crate::database::connection::DbPool;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Member,
    Treasurer
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub password_hash: String,
    pub user_role: UserRole,
    pub phone: Option<String>,
    pub dob: Option<DateTime<Utc>>,
    pub photo_url: Option<String>,
}

impl User {
    pub async fn create(pool: &DbPool, user: CreateUser) -> Result<Self, sqlx::Error> {
        let now = Utc::now();
        let hashed_password =
            hash(user.password_hash.as_bytes(), DEFAULT_COST).map_err(|_| sqlx::Error::RowNotFound)?;

        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (id, fullname, email, password_hash, user_role, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7) 
             RETURNING *",
        )
            .bind(Uuid::new_v4())
            .bind(user.fullname)
            .bind(user.email)
            .bind(hashed_password)
            .bind(user.user_role)
            .bind(now)
            .bind(now)
            .fetch_one(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_email(
        pool: &DbPool,
        email: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Self>, sqlx::Error> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
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
    ) -> Result<Option<Self>, sqlx::Error> {
        if let Some(user) = Self::find_by_email(pool, email).await? {
            if user.verify_password(password).unwrap_or(false) {
                return Ok(Some(user));
            }
        }
        Ok(None)
    }
}
