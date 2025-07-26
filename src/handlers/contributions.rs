use crate::{
    database::connection::DbPool,
    middleware::auth::AuthenticatedUser,
    models::contribution::{
        Contribution, ContributionError, CreateContribution, UpdateContribution,
    },
    requests::contribution::{ContributionRequest, UpdateContributionRequest},
    utils::helpers::ApiResponse,
};
use actix_web::{HttpResponse, Result, web};
use chrono::{DateTime, TimeZone, Utc};
use tracing::{error, info};
use uuid::Uuid;

pub async fn create(
    pool: web::Data<DbPool>,
    request: web::Json<ContributionRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    info!("Creating contribution for user: {}", user.user_id);

    let create_contribution = CreateContribution {
        title: request.title.clone(),
        description: request.description.clone(),
        amount: request.amount,
        due_date: request.due_date.unwrap(),
        created_by: user.user_id,
    };

    match Contribution::create(&pool, create_contribution).await {
        Ok(contribution) => {
            info!(
                "Successfully created contribution with ID: {}",
                contribution.id
            );
            Ok(HttpResponse::Created().json(ApiResponse::success(contribution)))
        }
        Err(ContributionError::Database(e)) => {
            error!("Database error creating contribution: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to create contribution".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error creating contribution: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn get_contribution(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let contribution_id = path.into_inner();
    info!("Getting contribution {}", contribution_id);

    match Contribution::find_by_id(&pool, contribution_id).await {
        Ok(Some(contribution)) => Ok(HttpResponse::Ok().json(ApiResponse::success(contribution))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Contribution not found".to_string(),
        ))),
        Err(ContributionError::Database(e)) => {
            error!("Database error getting contribution: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve contribution".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting contribution: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn get_user_contributions(
    pool: web::Data<DbPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    info!("Getting all contributions for user: {}", user.user_id);

    println!("{:?}", user);
    match Contribution::find_by_creator(&pool, user.user_id).await {
        Ok(contributions) => Ok(HttpResponse::Ok().json(ApiResponse::success(contributions))),
        Err(ContributionError::Database(e)) => {
            error!("Database error getting user contributions: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve contributions".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting user contributions: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn all(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    info!("Getting all contributions");

    match Contribution::find_all(&pool).await {
        Ok(contributions) => Ok(HttpResponse::Ok().json(ApiResponse::success(contributions))),
        Err(ContributionError::Database(e)) => {
            error!("Database error getting all contributions: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve contributions".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting all contributions: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn update(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateContributionRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let contribution_id = path.into_inner();
    info!(
        "Updating contribution {} for user: {}",
        contribution_id, user.user_id
    );

    match Contribution::find_by_id(&pool, contribution_id).await {
        Ok(Some(existing)) => {
            if existing.created_by != user.user_id {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Access denied".to_string())));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Contribution not found".to_string(),
            )));
        }
        Err(e) => {
            error!("Error checking contribution ownership: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to verify contribution".to_string(),
                )),
            );
        }
    }

    let update_data = UpdateContribution {
        title: request.title.clone(),
        description: request.description.clone(),
        amount: request.amount,
        due_date: request.due_date,
    };

    match Contribution::update(&pool, contribution_id, update_data).await {
        Ok(contribution) => {
            info!("Successfully updated contribution: {}", contribution_id);
            Ok(HttpResponse::Ok().json(ApiResponse::success(contribution)))
        }
        Err(ContributionError::NotFound { id }) => Ok(HttpResponse::NotFound().json(
            ApiResponse::<()>::error(format!("Contribution {} not found", id)),
        )),
        Err(ContributionError::NoUpdateFields) => Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error("No fields provided for update".to_string()),
        )),
        Err(ContributionError::Database(e)) => {
            error!("Database error updating contribution: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to update contribution".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error updating contribution: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn delete(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let contribution_id = path.into_inner();
    info!(
        "Deleting contribution {} for user: {}",
        contribution_id, user.user_id
    );

    match Contribution::find_by_id(&pool, contribution_id).await {
        Ok(Some(existing)) => {
            if existing.created_by != user.user_id {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Access denied".to_string())));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Contribution not found".to_string(),
            )));
        }
        Err(e) => {
            error!("Error checking contribution ownership: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to verify contribution".to_string(),
                )),
            );
        }
    }

    match Contribution::delete(&pool, contribution_id).await {
        Ok(()) => {
            info!("Successfully deleted contribution: {}", contribution_id);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::success(())))
        }
        Err(ContributionError::NotFound { id }) => Ok(HttpResponse::NotFound().json(
            ApiResponse::<()>::error(format!("Contribution {} not found", id)),
        )),
        Err(ContributionError::Database(e)) => {
            error!("Database error deleting contribution: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to delete contribution".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error deleting contribution: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}
