use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use dino_park_gate::scope::ScopeAndUser;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct TermsUpdate {
    text: String,
}

fn view_terms(pool: web::Data<Pool>, group_name: web::Path<String>) -> impl Responder {
    match operations::terms::get_terms(&pool, &group_name) {
        Ok(terms) => Ok(HttpResponse::Ok().json(terms)),
        Err(e) => Err(ApiError::NotAcceptableError(e)),
    }
}

fn delete_terms(
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    match operations::terms::delete_terms(&pool, &scope_and_user, &group_name) {
        Ok(_) => Ok(HttpResponse::Created().finish()),
        Err(e) => Err(ApiError::NotAcceptableError(e)),
    }
}

fn update_terms(
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    terms_update: web::Json<TermsUpdate>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    match operations::terms::update_terms(
        &pool,
        &scope_and_user,
        &group_name,
        terms_update.into_inner().text,
    ) {
        Ok(_) => Ok(HttpResponse::Created().finish()),
        Err(e) => Err(ApiError::NotAcceptableError(e)),
    }
}

pub fn terms_app() -> impl HttpServiceFactory {
    web::scope("/terms")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(
            web::resource("/{group_name}")
                .route(web::get().to(view_terms))
                .route(web::put().to(update_terms))
                .route(web::delete().to(delete_terms))
                .route(web::post().to(update_terms)),
        )
}
