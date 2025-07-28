use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}
