use crate::models::photo::{CreatePhoto, Photo, PhotoError, UpdatePhoto};
use crate::models::user::UserRole;
use crate::requests::photo::{CreatePhotoRequest, UpdatePhotoRequest};
use crate::{
    database::connection::DbPool, middleware::auth::AuthenticatedUser, utils::helpers::ApiResponse,
};
use actix_web::{HttpResponse, Result, web};
use chrono::{DateTime, TimeZone, Utc};
use tracing::{error, info};
use uuid::Uuid;

pub async fn create(
    pool: web::Data<DbPool>,
    request: web::Json<CreatePhotoRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    info!("Creating photo for user: {}", user.user_id);

    let create_photo = CreatePhoto {
        caption: request.caption.clone(),
        url: request.url.clone(),
        event_id: request.event_id.clone(),
        posted_by: user.user_id,
    };

    match Photo::create(&pool, create_photo).await {
        Ok(photo) => {
            info!("Successfully created photo with ID: {}", photo.id);
            Ok(HttpResponse::Created().json(ApiResponse::success(photo)))
        }
        Err(PhotoError::Database(e)) => {
            error!("Database error creating photo: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to create photo".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error creating photo: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn get_photo(pool: web::Data<DbPool>, path: web::Path<Uuid>) -> Result<HttpResponse> {
    let photo_id = path.into_inner();
    info!("Getting photo {}", photo_id);

    match Photo::find_by_id(&pool, photo_id).await {
        Ok(Some(photo)) => Ok(HttpResponse::Ok().json(ApiResponse::success(photo))),
        Ok(None) => {
            Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Photo not found".to_string())))
        }
        Err(PhotoError::Database(e)) => {
            error!("Database error getting photo: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve photo".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting photo: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn all(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    info!("Getting all photos");

    match Photo::find_all(&pool).await {
        Ok(photos) => Ok(HttpResponse::Ok().json(ApiResponse::success(photos))),
        Err(PhotoError::Database(e)) => {
            error!("Database error getting all photos: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve photos".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting all photos: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn update(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    request: web::Json<UpdatePhotoRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let photo_id = path.into_inner();
    info!("Updating photo {} for user: {}", photo_id, user.user_id);

    match Photo::find_by_id(&pool, photo_id).await {
        Ok(Some(existing)) => {
            if existing.posted_by != user.user_id
                && user.user_role != UserRole::Admin
                && user.user_role != UserRole::SuperAdmin
            {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Access denied".to_string())));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Photo not found".to_string())));
        }
        Err(e) => {
            error!("Error checking photo ownership: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to verify photo".to_string(),
                )),
            );
        }
    }

    let update_data = UpdatePhoto {
        caption: Some(request.caption.clone()),
        url: request.url.clone(),
        event_id: Some(request.event_id.clone()),
    };

    match Photo::update(&pool, photo_id, update_data).await {
        Ok(photo) => {
            info!("Successfully updated photo: {}", photo_id);
            Ok(HttpResponse::Ok().json(ApiResponse::success(photo)))
        }
        Err(PhotoError::NotFound { id }) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::<()>::error(format!("Photo {} not found", id)))),
        Err(PhotoError::NoUpdateFields) => Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error("No fields provided for update".to_string()),
        )),
        Err(PhotoError::Database(e)) => {
            error!("Database error updating photo: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to update photo".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error updating photo: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn delete(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let photo_id = path.into_inner();
    info!("Deleting photo {} for user: {}", photo_id, user.user_id);

    match Photo::find_by_id(&pool, photo_id).await {
        Ok(Some(existing)) => {
            if existing.posted_by != user.user_id
                && user.user_role != UserRole::Admin
                && user.user_role != UserRole::SuperAdmin
            {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Access denied".to_string())));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Photo not found".to_string())));
        }
        Err(e) => {
            error!("Error checking photo ownership: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to verify photo".to_string(),
                )),
            );
        }
    }

    match Photo::delete(&pool, photo_id).await {
        Ok(()) => {
            info!("Successfully deleted photo: {}", photo_id);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::success(())))
        }
        Err(PhotoError::NotFound { id }) => Ok(HttpResponse::NotFound()
            .json(ApiResponse::<()>::error(format!("Photo {} not found", id)))),
        Err(PhotoError::Database(e)) => {
            error!("Database error deleting photo: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to delete photo".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error deleting photo: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}
