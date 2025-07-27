use crate::models::user::UserRole;
use crate::services::auth::AuthService;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::ErrorUnauthorized,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub user_role: UserRole,
}

impl AuthenticatedUser {
    pub fn new(user_id: Uuid, email: String, user_role: UserRole) -> Self {
        Self {
            user_id,
            email,
            user_role,
        }
    }
}

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Extract the Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            let token = match auth_header {
                Some(token) => token,
                None => {
                    return Err(ErrorUnauthorized("Missing or invalid authorization header"));
                }
            };

            // Validate the token
            let auth_service = AuthService::new()
                .map_err(|_| ErrorUnauthorized("Authentication service error"))?;

            let claims = auth_service
                .validate_token(token)
                .map_err(|_| ErrorUnauthorized("Invalid token"))?;

            // Create authenticated user and add to request extensions
            let authenticated_user =
                AuthenticatedUser::new(claims.sub, claims.email, claims.user_role);
            req.extensions_mut().insert(authenticated_user);

            // Continue with the request
            service.call(req).await
        })
    }
}

impl actix_web::FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = futures_util::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        match req.extensions().get::<AuthenticatedUser>() {
            Some(user) => futures_util::future::ready(Ok(user.clone())),
            None => futures_util::future::ready(Err(ErrorUnauthorized("User not authenticated"))),
        }
    }
}
