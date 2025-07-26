use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ContributionRequest {
    pub title: String,
    pub description: Option<String>,
    pub amount: Option<Decimal>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContributionRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub amount: Option<Decimal>,
    pub due_date: Option<NaiveDate>,
}