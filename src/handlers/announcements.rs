use crate::models::announcement::{
    Announcement, AnnouncementError, CreateAnnouncement, UpdateAnnouncement,
};
use crate::requests::announcement::{CreateAnnouncementRequest, UpdateAnnouncementRequest};
use crate::{
    database::connection::DbPool, middleware::auth::AuthenticatedUser, utils::helpers::ApiResponse,
};
use actix_web::{HttpResponse, Result, web};
use chrono::{DateTime, TimeZone, Utc};
use tracing::{error, info};
use uuid::Uuid;

pub async fn create(
    pool: web::Data<DbPool>,
    request: web::Json<CreateAnnouncementRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    info!("Creating announcement for user: {}", user.user_id);

    let create_announcement = CreateAnnouncement {
        title: request.title.clone(),
        body: request.body.clone(),
        posted_by: user.user_id,
    };

    match Announcement::create(&pool, create_announcement).await {
        Ok(announcement) => {
            info!(
                "Successfully created announcement with ID: {}",
                announcement.id
            );
            Ok(HttpResponse::Created().json(ApiResponse::success(announcement)))
        }
        Err(AnnouncementError::Database(e)) => {
            error!("Database error creating announcement: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to create announcement".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error creating announcement: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn get_announcement(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let announcement_id = path.into_inner();
    info!("Getting announcement {}", announcement_id);

    match Announcement::find_by_id(&pool, announcement_id).await {
        Ok(Some(announcement)) => Ok(HttpResponse::Ok().json(ApiResponse::success(announcement))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Announcement not found".to_string(),
        ))),
        Err(AnnouncementError::Database(e)) => {
            error!("Database error getting announcement: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve announcement".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting announcement: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn all(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    info!("Getting all announcements");

    match Announcement::find_all(&pool).await {
        Ok(announcements) => Ok(HttpResponse::Ok().json(ApiResponse::success(announcements))),
        Err(AnnouncementError::Database(e)) => {
            error!("Database error getting all announcements: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to retrieve announcements".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error getting all announcements: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn update(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateAnnouncementRequest>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let announcement_id = path.into_inner();
    info!(
        "Updating announcement {} for user: {}",
        announcement_id, user.user_id
    );

    match Announcement::find_by_id(&pool, announcement_id).await {
        Ok(Some(existing)) => {
            if existing.posted_by != user.user_id {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Access denied".to_string())));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Announcement not found".to_string(),
            )));
        }
        Err(e) => {
            error!("Error checking announcement ownership: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to verify announcement".to_string(),
                )),
            );
        }
    }

    let update_data = UpdateAnnouncement {
        title: request.title.clone(),
        body: request.body.clone(),
    };

    match Announcement::update(&pool, announcement_id, update_data).await {
        Ok(announcement) => {
            info!("Successfully updated announcement: {}", announcement_id);
            Ok(HttpResponse::Ok().json(ApiResponse::success(announcement)))
        }
        Err(AnnouncementError::NotFound { id }) => Ok(HttpResponse::NotFound().json(
            ApiResponse::<()>::error(format!("Announcement {} not found", id)),
        )),
        Err(AnnouncementError::NoUpdateFields) => Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error("No fields provided for update".to_string()),
        )),
        Err(AnnouncementError::Database(e)) => {
            error!("Database error updating announcement: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to update announcement".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error updating announcement: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}

pub async fn delete(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let announcement_id = path.into_inner();
    info!(
        "Deleting announcement {} for user: {}",
        announcement_id, user.user_id
    );

    match Announcement::find_by_id(&pool, announcement_id).await {
        Ok(Some(existing)) => {
            if existing.posted_by != user.user_id {
                return Ok(HttpResponse::Forbidden()
                    .json(ApiResponse::<()>::error("Access denied".to_string())));
            }
        }
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Announcement not found".to_string(),
            )));
        }
        Err(e) => {
            error!("Error checking announcement ownership: {}", e);
            return Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to verify announcement".to_string(),
                )),
            );
        }
    }

    match Announcement::delete(&pool, announcement_id).await {
        Ok(()) => {
            info!("Successfully deleted announcement: {}", announcement_id);
            Ok(HttpResponse::Ok().json(ApiResponse::<()>::success(())))
        }
        Err(AnnouncementError::NotFound { id }) => Ok(HttpResponse::NotFound().json(
            ApiResponse::<()>::error(format!("Announcement {} not found", id)),
        )),
        Err(AnnouncementError::Database(e)) => {
            error!("Database error deleting announcement: {}", e);
            Ok(
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    "Failed to delete announcement".to_string(),
                )),
            )
        }
        Err(e) => {
            error!("Error deleting announcement: {}", e);
            Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(e.to_string())))
        }
    }
}
