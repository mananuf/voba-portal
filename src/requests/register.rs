use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub fullname: String,
    pub email: String,
    pub password: String,
    pub user_role: Option<String>,
}