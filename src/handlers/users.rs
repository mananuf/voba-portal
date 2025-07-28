use crate::{database::connection::DbPool, models::user::User, utils::helpers::ApiResponse};
use actix_web::{HttpResponse, Result, web};
use tracing::error;
use tracing::log::info;
use uuid::Uuid;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::user::{UserError, UserRole};

pub async fn index(pool: web::Data<DbPool>) -> Result<HttpResponse> {
    let users = User::find_all(&pool).await.map_err(|e| {
        error!("Failed to fetch users: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch users")
    })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(users)))
}

pub async fn toggle_user_active(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let target_user_id = path.into_inner();
    info!("Toggling active status for user: {}", target_user_id);

    if user.user_role != UserRole::SuperAdmin && user.user_role != UserRole::Admin {
        return Ok(HttpResponse::Forbidden().json(
            ApiResponse::<()>::error("You don't have permission to perform this action".to_string())
        ));
    }

    if target_user_id == user.user_id {
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error("You cannot deactivate your own account".to_string())
        ));
    }

    match User::toggle_active(&pool, target_user_id).await {
        Ok(updated_user) => {
            info!(
                "User {} active status toggled to {} by {}",
                target_user_id, updated_user.is_active, user.user_id
            );
            Ok(HttpResponse::Ok().json(ApiResponse::success(updated_user)))
        }
        Err(UserError::NotFound { id }) => Ok(HttpResponse::NotFound().json(
            ApiResponse::<()>::error(format!("User {} not found", id))
        )),
        Err(e) => {
            error!("Failed to toggle active status: {}", e);
            Ok(HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("Failed to update user status".to_string())
            ))
        }
    }
}