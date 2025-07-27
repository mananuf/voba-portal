use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreatePhotoRequest {
    pub event_id: Option<Uuid>,
    pub url: String,
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePhotoRequest {
    pub event_id: Option<Uuid>,
    pub url: Option<String>,
    pub caption: Option<String>,
}
