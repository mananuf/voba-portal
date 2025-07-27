use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnnouncementRequest {
    pub posted_by: Uuid,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAnnouncementRequest {
    pub title: Option<String>,
    pub body: Option<String>,
}
