use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

impl ApiResponse<()> {
    pub fn to_response(&self, status: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(self)
    }
}
