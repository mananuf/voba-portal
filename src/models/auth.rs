use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::user::UserRole;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub fullname: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub user_role: UserRole,
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, user_role: UserRole) -> Self {
        let now = Utc::now().timestamp();
        Self {
            sub: user_id,
            email: email,
            user_role: user_role,
            exp: now + (24 * 60 * 60), // 24 hours
            iat: now,
        }
    }
}
