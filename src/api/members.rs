use crate::db::db::Pool;
use crate::db::operations;
use crate::db::types::RoleType;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use serde_derive::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct GetMembersQuery {
    next: Option<i64>,
}

fn get_members(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    query: web::Query<GetMembersQuery>,
) -> impl Responder {
    let page_size = 20;
    let next = query.next;
    match operations::members::scoped_members_and_host(
        &*pool,
        &*group_name,
        &scope_and_user.scope,
        &[RoleType::Admin, RoleType::Curator, RoleType::Member],
        page_size,
        next,
    ) {
        Ok(members) => Ok(HttpResponse::Ok().json(members)),
        Err(_) => Err(error::ErrorNotFound("")),
    }
}

pub fn members_app() -> impl HttpServiceFactory {
    web::scope("/members")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("/{group_name}").route(web::get().to(get_members)))
}
