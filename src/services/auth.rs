use crate::database::connection::DbPool;
use crate::models::auth::Claims;
use crate::models::user::User;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::env;

pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        })
    }

    pub fn generate_token(&self, user: &User) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims::new(user.id, user.email.clone());
        encode(&Header::default(), &claims, &self.encoding_key)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(token_data.claims)
    }

    pub async fn authenticate_user(
        &self,
        pool: &DbPool,
        email: &str,
        password: &str,
    ) -> Result<Option<User>, Box<dyn std::error::Error>> {
        User::authenticate(pool, email, password)
            .await
            .map_err(|e| e.into())
    }
}
