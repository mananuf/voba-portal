use crate::database::connection::DbPool;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContributionError {
    #[error("Contribution with ID {id} not found")]
    NotFound { id: Uuid },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("No fields provided for update")]
    NoUpdateFields,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contribution {
    pub id: Uuid,
    pub created_by: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub due_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContribution {
    pub created_by: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub due_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContribution {
    pub title: Option<String>,
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub due_date: Option<DateTime<Utc>>,
}

impl Contribution {
    pub async fn create(pool: &DbPool, contribution: CreateContribution) -> Result<Self, sqlx::Error> {
        let now = Utc::now();

        let contribution = sqlx::query_as::<_, Contribution>(
            "INSERT INTO contributions (id, title, description, amount, due_date, created_by, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
             RETURNING *",
        )
            .bind(Uuid::new_v4())
            .bind(contribution.title)
            .bind(contribution.description.unwrap_or("".to_string()))
            .bind(contribution.amount.unwrap_or(0.0))
            .bind(contribution.due_date)
            .bind(contribution.created_by)
            .bind(now)
            .bind(now)
            .fetch_one(pool)
            .await?;

        Ok(contribution)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        let contribution = sqlx::query_as::<_,Contribution>("SELECT * FROM contributions WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(contribution)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Self>, sqlx::Error> {
        let contributions = sqlx::query_as::<_, Contribution>("SELECT * FROM contributions ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?;

        Ok(contributions)
    }

    pub async fn update(pool: &DbPool, id: Uuid, update_data: UpdateContribution) -> Result<Option<Self>, ContributionError> {
        if update_data.title.is_none()
            && update_data.description.is_none()
            && update_data.amount.is_none()
            && update_data.due_date.is_none() {
            return Err(ContributionError::NoUpdateFields);
        }
        
        let existing = match Self::find_by_id(pool, id).await? {
            Some(contribution) => contribution,
            None => return Err(ContributionError::NotFound { id }),
        };

        let now = Utc::now();

        let updated_contribution = sqlx::query_as::<_, Contribution>(
            "UPDATE contributions 
             SET title = $2, description = $3, amount = $4, due_date = $5, updated_at = $6
             WHERE id = $1 
             RETURNING *",
        )
            .bind(id)
            .bind(update_data.title.unwrap_or(existing.title))
            .bind(update_data.description.or(existing.description))
            .bind(update_data.amount.or(existing.amount))
            .bind(update_data.due_date.unwrap_or(existing.due_date))
            .bind(now)
            .fetch_optional(pool)
            .await?;

        Ok(updated_contribution)
    }

    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool, ContributionError> {
        let result = sqlx::query("DELETE FROM contributions WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ContributionError::NotFound { id });
        }

        Ok(true)
    }

    pub async fn find_by_creator(pool: &DbPool, created_by: Uuid) -> Result<Vec<Self>, sqlx::Error> {
        let contributions = sqlx::query_as::<_, Contribution>(
            "SELECT * FROM contributions WHERE created_by = $1 ORDER BY created_at DESC"
        )
            .bind(created_by)
            .fetch_all(pool)
            .await?;

        Ok(contributions)
    }
    
    pub async fn find_due_before(pool: &DbPool, before_date: DateTime<Utc>) -> Result<Vec<Self>, sqlx::Error> {
        let contributions = sqlx::query_as::<_, Contribution>(
            "SELECT * FROM contributions WHERE due_date <= $1 ORDER BY due_date ASC"
        )
            .bind(before_date)
            .fetch_all(pool)
            .await?;

        Ok(contributions)
    }
}
