use actix_web::{HttpResponse, web};

use crate::handlers;
use crate::middleware::auth::AuthMiddleware;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(web::resource("/register").route(web::post().to(handlers::auth::register)))
            .service(web::resource("/login").route(web::post().to(handlers::auth::login))),
    )
    .service(
        web::scope("/users").service(
            web::resource("")
                .route(web::get().to(handlers::users::index))
                .route(web::head().to(HttpResponse::MethodNotAllowed)),
        ),
    )
    .service(
        web::scope("/contributions")
            .service(
                web::resource("")
                    .route(
                        web::post()
                            .to(handlers::contributions::create)
                            .wrap(AuthMiddleware),
                    )
                    .route(web::get().to(handlers::contributions::all))
                    .route(web::head().to(HttpResponse::MethodNotAllowed)),
            )
            .service(
                web::resource("/user")
                    .route(
                        web::get()
                            .to(handlers::contributions::get_user_contributions)
                            .wrap(AuthMiddleware),
                    )
                    .route(web::head().to(HttpResponse::MethodNotAllowed)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(handlers::contributions::get_contribution))
                    .route(
                        web::delete()
                            .to(handlers::contributions::delete)
                            .wrap(AuthMiddleware),
                    )
                    .route(
                        web::put()
                            .to(handlers::contributions::update)
                            .wrap(AuthMiddleware),
                    )
                    .route(web::head().to(HttpResponse::MethodNotAllowed)),
            ),
    );
}
