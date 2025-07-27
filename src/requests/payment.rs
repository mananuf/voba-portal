use crate::models::payment::PaymentStatus;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PaymentRequest {
    pub user_id: Uuid,
    pub contribution_id: Uuid,
    pub amount: Option<Decimal>,
    pub receipt_url: Option<String>,
    pub status: PaymentStatus,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePaymentRequest {
    pub user_id: Option<Uuid>,
    pub contribution_id: Option<Uuid>,
    pub amount: Option<Decimal>,
    pub receipt_url: Option<String>,
    pub status: Option<PaymentStatus>,
}
