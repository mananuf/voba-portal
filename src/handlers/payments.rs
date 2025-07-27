use crate::{
    database::connection::DbPool,
    middleware::auth::AuthenticatedUser,
    utils::helpers::ApiResponse,
};
use actix_web::{HttpResponse, Result, web};
use chrono::{DateTime, TimeZone, Utc};
use tracing::{error, info};
use uuid::Uuid;
use crate::models::payment::{CreatePayment, Payment, PaymentError, PaymentStatus, UpdatePayment};
use crate::models::user::UserRole;
use crate::requests::payment::{PaymentRequest, UpdatePaymentRequest};

pub async fn create(
    pool: web::Data<DbPool>,
    request: web::Json<PaymentRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    info!("Creating contribution for user: {}", user.user_id);

    let create_payment = CreatePayment {
        user_id: request.user_id.clone(),
        contribution_id: request.contribution_id.clone(),
        amount: request.amount.clone(),
        receipt_url: request.receipt_url.clone(),
        status: request.status.clone(),
    };

    match Payment::create(&pool, create_payment).await {
        Ok(payment) => {
            info!(
                "Successfully created payment with ID: {}",
                payment.id
            );
            Ok(HttpResponse::Created().json(ApiResponse::success(payment)))
        }
        Err(PaymentError::Database(e)) => {
            error!("Database error creating payment: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to create payment".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error creating payment: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn get_payment(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let payment_id = path.into_inner();
    info!("Getting contribution {}", payment_id);

    match Payment::find_by_id(&pool, payment_id).await {
        Ok(Some(payment)) => Ok(HttpResponse::Ok().json(ApiResponse::success(payment))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Payment not found".to_string(),
        ))),
        Err(PaymentError::Database(e)) => {
            error!("Database error getting payment: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve payment".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting payment: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn get_user_payments(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    info!("Getting all contributions for user: {}", user_id);
    
    match Payment::find_by_user(&pool, *user_id).await {
        Ok(payments) => Ok(HttpResponse::Ok().json(ApiResponse::success(payments))),
        Err(PaymentError::Database(e)) => {
            error!("Database error getting user payments: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve payments".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting user payments: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn all(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    info!("Getting all payments");

    match Payment::find_all(&pool).await {
        Ok(payments) => Ok(HttpResponse::Ok().json(ApiResponse::success(payments))),
        Err(PaymentError::Database(e)) => {
            error!("Database error getting all payments: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve payments".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting all payments: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn update(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    request: web::Json<UpdatePaymentRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let payment_id = path.into_inner();
    info!(
        "Updating payment {} for user: {}",
        payment_id, user.user_id
    );
    
    match Payment::find_by_id(&pool, payment_id).await {
        Ok(Some(existing)) => {
            if existing.user_id != user.user_id && user.user_role != UserRole::Admin && user.user_role != UserRole::SuperAdmin {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Access denied".to_string())));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Payment not found".to_string(),
            )));
        }
        Err(e) => {
            error!("Error checking payment ownership: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to verify payment".to_string(),
                )),
            );
        }
    }

    let update_data = UpdatePayment {
        user_id: request.user_id.clone(),
        contribution_id: request.contribution_id.clone(),
        amount: request.amount.clone(),
        receipt_url: request.receipt_url.as_ref().map(|url| Some(url.clone())),
        status: request.status.clone(),
    };

    match Payment::update(&pool, payment_id, update_data).await {
        Ok(payment) => {
            info!("Successfully updated payment: {}", payment_id);
            Ok(HttpResponse::Ok().json(ApiResponse::success(payment)))
        }
        Err(PaymentError::NotFound { id }) => Ok(HttpResponse::NotFound().json(
            ApiResponse::<()>::error(format!("Payment {} not found", id)),
        )),
        Err(PaymentError::NoUpdateFields) => Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error("No fields provided for update".to_string()),
        )),
        Err(PaymentError::Database(e)) => {
            error!("Database error updating payment: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to update payment".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error updating payment: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn delete(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let payment_id = path.into_inner();
    info!(
        "Deleting payment {} for user: {}",
        payment_id, user.user_id
    );

    match Payment::find_by_id(&pool, payment_id).await {
        Ok(Some(existing)) => {
            if user.user_role != UserRole::Admin && user.user_role != UserRole::SuperAdmin {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Access denied".to_string())));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Payment not found".to_string(),
            )));
        }
        Err(e) => {
            error!("Error checking payment ownership: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to verify payment".to_string(),
                )),
            );
        }
    }

    match Payment::delete(&pool, payment_id).await {
        Ok(()) => {
            info!("Successfully deleted payment: {}", payment_id);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::success(())))
        }
        Err(PaymentError::NotFound { id }) => Ok(HttpResponse::NotFound().json(
            ApiResponse::<()>::error(format!("Payment {} not found", id)),
        )),
        Err(PaymentError::Database(e)) => {
            error!("Database error deleting payment: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to delete payment".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error deleting payment: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}
