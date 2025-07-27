use crate::database::connection::DbPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AnnouncementError {
    #[error("Announcement with ID {id} not found")]
    NotFound { id: Uuid },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("No fields provided for update")]
    NoUpdateFields,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Announcement {
    pub id: Uuid,
    pub posted_by: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnnouncement {
    pub posted_by: Uuid,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAnnouncement {
    pub title: Option<String>,
    pub body: Option<String>,
}

impl Announcement {
    pub async fn create(
        pool: &DbPool,
        announcement: CreateAnnouncement,
    ) -> Result<Self, AnnouncementError> {
        let now = Utc::now();

        let announcement = sqlx::query_as::<_, Announcement>(
            "INSERT INTO announcements (id, title, body, posted_by, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(announcement.title)
        .bind(announcement.body)
        .bind(announcement.posted_by)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(announcement)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, AnnouncementError> {
        let announcement =
            sqlx::query_as::<_, Announcement>("SELECT * FROM announcements WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?;

        Ok(announcement)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Self>, AnnouncementError> {
        let announcements = sqlx::query_as::<_, Announcement>(
            "SELECT * FROM announcements ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(announcements)
    }

    pub async fn update(
        pool: &DbPool,
        id: Uuid,
        update_data: UpdateAnnouncement,
    ) -> Result<Option<Self>, AnnouncementError> {
        if update_data.title.is_none() && update_data.body.is_none() {
            return Err(AnnouncementError::NoUpdateFields);
        }

        let existing = match Self::find_by_id(pool, id).await? {
            Some(announcement) => announcement,
            None => return Err(AnnouncementError::NotFound { id }),
        };

        let now = Utc::now();

        let updated_announcement = sqlx::query_as::<_, Announcement>(
            "UPDATE announcements 
             SET title = $2, body = $3, updated_at = $4
             WHERE id = $1 
             RETURNING *",
        )
        .bind(id)
        .bind(update_data.title.unwrap_or(existing.title))
        .bind(update_data.body.or(existing.body))
        .bind(now)
        .fetch_optional(pool)
        .await?;

        Ok(updated_announcement)
    }

    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<(), AnnouncementError> {
        let result = sqlx::query("DELETE FROM announcements WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AnnouncementError::NotFound { id });
        }

        Ok(())
    }
}
