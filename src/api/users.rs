use crate::db::db::Pool;
use crate::db::operations;
use crate::db::types::TrustType;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use dino_park_gate::scope::ScopeAndUser;
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct SearchUsersQuery {
    q: String,
    t: Option<TrustType>,
}
fn search_users(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    query: web::Query<SearchUsersQuery>,
) -> impl Responder {
    match operations::users::search_users(&pool, scope_and_user, query.t.clone(), &query.q) {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(_) => Err(error::ErrorNotFound("")),
    }
}

pub fn users_app() -> impl HttpServiceFactory {
    web::scope("/users")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("").route(web::get().to(search_users)))
}
