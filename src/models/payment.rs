use crate::database::connection::DbPool;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("Payment with ID {id} not found")]
    NotFound { id: Uuid },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("No fields provided for update")]
    NoUpdateFields,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Verified,
}

impl FromStr for PaymentStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(PaymentStatus::Pending),
            "verified" => Ok(PaymentStatus::Verified),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub contribution_id: Uuid,
    pub amount: Option<Decimal>,
    pub receipt_url: Option<String>,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePayment {
    pub user_id: Uuid,
    pub contribution_id: Uuid,
    pub amount: Option<Decimal>,
    pub receipt_url: Option<String>,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePayment {
    pub user_id: Option<Uuid>,
    pub contribution_id: Option<Uuid>,
    pub amount: Option<Decimal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_url: Option<Option<String>>,
    pub status: Option<PaymentStatus>,
}

impl Payment {
    pub async fn create(pool: &DbPool, payment: CreatePayment) -> Result<Self, PaymentError> {
        let now = Utc::now();

        let payment = sqlx::query_as::<_, Payment>(
            "INSERT INTO payments (id, user_id, contribution_id, amount, receipt_url, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING *",
        )
            .bind(Uuid::new_v4())
            .bind(payment.user_id)
            .bind(payment.contribution_id)
            .bind(payment.amount)
            .bind(payment.receipt_url)
            .bind(payment.status)
            .bind(now)
            .bind(now)
            .fetch_one(pool)
            .await?;

        Ok(payment)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Self>, PaymentError> {
        let payment = sqlx::query_as::<_, Payment>("SELECT * FROM payments WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(payment)
    }

    pub async fn find_by_user(pool: &DbPool, user_id: Uuid) -> Result<Option<Self>, PaymentError> {
        let payments = sqlx::query_as::<_, Payment>("SELECT * FROM payments WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

        Ok(payments)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Self>, PaymentError> {
        let payments =
            sqlx::query_as::<_, Payment>("SELECT * FROM payments ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?;

        Ok(payments)
    }

    pub async fn update(
        pool: &DbPool,
        id: Uuid,
        update_data: UpdatePayment,
    ) -> Result<Option<Self>, PaymentError> {
        if update_data.user_id.is_none()
            && update_data.contribution_id.is_none()
            && update_data.amount.is_none()
            && update_data.status.is_none()
            && update_data.receipt_url.is_none()
        {
            return Err(PaymentError::NoUpdateFields);
        }

        let existing = match Self::find_by_id(pool, id).await? {
            Some(payment) => payment,
            None => return Err(PaymentError::NotFound { id }),
        };

        let now = Utc::now();

        let updated_payment = sqlx::query_as::<_, Payment>(
            r#"
            UPDATE payments
            SET 
                user_id = COALESCE($2, user_id),
                contribution_id = COALESCE($3, contribution_id),
                amount = COALESCE($4, amount),
                status = COALESCE($5, status),
                receipt_url = COALESCE($6, receipt_url),
                updated_at = $7
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(update_data.user_id.unwrap_or(existing.user_id))
        .bind(
            update_data
                .contribution_id
                .or(Some(existing.contribution_id)),
        )
        .bind(update_data.amount.or(existing.amount))
        .bind(update_data.status.unwrap_or(existing.status))
        .bind(update_data.receipt_url.unwrap_or_default())
        .bind(now)
        .fetch_optional(pool)
        .await?;

        Ok(updated_payment)
    }

    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<(), PaymentError> {
        let result = sqlx::query("DELETE FROM payments WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(PaymentError::NotFound { id });
        }

        Ok(())
    }
}
