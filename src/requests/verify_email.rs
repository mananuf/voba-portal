use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    pub code: String,
}