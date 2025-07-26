use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;
use crate::models::payment::PaymentStatus;

#[derive(Debug, Deserialize)]
pub struct PaymentRequest {
    pub user_id: Uuid,
    pub contribution_id: Uuid,
    pub amount: Option<Decimal>,
    pub receipt_url: Option<String>,
    pub status: PaymentStatus,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentRequest {
    pub user_id: Option<Uuid>,
    pub contribution_id: Option<Uuid>,
    pub amount: Option<Decimal>,
    pub receipt_url: Option<String>,
    pub status: Option<PaymentStatus>,
}