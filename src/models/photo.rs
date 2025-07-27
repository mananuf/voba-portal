use crate::database::connection::DbPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum PhotoError {
    #[error("Photo with ID {id} not found")]
    NotFound { id: Uuid },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("No fields provided for update")]
    NoUpdateFields,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Photo {
    pub id: Uuid,
    pub posted_by: Uuid,
    pub event_id: Option<Uuid>,
    pub url: String,
    pub caption: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePhoto {
    pub posted_by: Uuid,
    pub event_id: Option<Uuid>,
    pub url: String,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePhoto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<Option<Uuid>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<Option<String>>,
}

impl Photo {
    pub async fn create(pool: &DbPool, photo: CreatePhoto) -> Result<Self, PhotoError> {
        let now = Utc::now();

        let photo = sqlx::query_as::<_, Photo>(
            "INSERT INTO photos (id, caption, event_id, url, posted_by, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7) 
             RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(photo.caption)
        .bind(photo.event_id)
        .bind(photo.url)
        .bind(photo.posted_by)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(photo)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, PhotoError> {
        let photo = sqlx::query_as::<_, Photo>("SELECT * FROM photos WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(photo)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Self>, PhotoError> {
        let photos = sqlx::query_as::<_, Photo>("SELECT * FROM photos ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?;

        Ok(photos)
    }

    pub async fn update(
        pool: &DbPool,
        id: Uuid,
        update_data: UpdatePhoto,
    ) -> Result<Option<Self>, PhotoError> {
        if update_data.caption.is_none()
            && update_data.event_id.is_none()
            && update_data.url.is_none()
        {
            return Err(PhotoError::NoUpdateFields);
        }

        let existing = match Self::find_by_id(pool, id).await? {
            Some(photo) => photo,
            None => return Err(PhotoError::NotFound { id }),
        };

        let now = Utc::now();

        let updated_photo = sqlx::query_as::<_, Photo>(
            "UPDATE photos 
             SET caption = $2, url = $3, event_id = $4, updated_at = $5
             WHERE id = $1 
             RETURNING *",
        )
        .bind(id)
        .bind(update_data.caption.unwrap_or_default())
        .bind(update_data.url.unwrap_or(existing.url))
        .bind(update_data.event_id.unwrap_or_default())
        .bind(now)
        .fetch_optional(pool)
        .await?;

        Ok(updated_photo)
    }

    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<(), PhotoError> {
        let result = sqlx::query("DELETE FROM photos WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(PhotoError::NotFound { id });
        }

        Ok(())
    }
}
