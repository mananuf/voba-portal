use actix_web::{web, HttpResponse};

use crate::handlers;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    //     .service(
    //     web::scope("/auth")
    //         .service(web::resource("/register").route(web::post().to(handlers::auth::register)))
    //         .service(web::resource("/login").route(web::post().to(handlers::auth::login))),
    // )
        .service(
            web::scope("/users").service(
                web::resource("")
                    .route(web::get().to(handlers::users::index))
                    .route(web::head().to(HttpResponse::MethodNotAllowed)),
            ),
        );
}
