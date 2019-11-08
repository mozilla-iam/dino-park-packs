use crate::db::db::Pool;
use crate::db::operations;
use crate::db::types::RoleType;
use crate::error::PacksError;
use crate::user::User;
use crate::utils::to_expiration_ts;
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
use futures::Future;
use serde_derive::Deserialize;
use serde_humantime::De;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GetMembersQuery {
    next: Option<i64>,
    size: Option<i64>,
}

fn group_details(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    query: web::Query<GetMembersQuery>,
) -> impl Responder {
    let page_size = query.size.unwrap_or_else(|| 20);
    let member_count = match operations::members::member_count(&*pool, &*group_name) {
        Ok(member_count) => member_count,
        _ => return Err(error::ErrorNotFound("")),
    };
    let group = operations::groups::get_group(&pool, &group_name)?;
    let curators = operations::members::scoped_members_and_host(
        &*pool,
        &*group_name,
        &scope_and_user.scope,
        &[RoleType::Admin, RoleType::Curator],
        page_size,
        None,
    )?;
    let members = operations::members::scoped_members_and_host(
        &*pool,
        &*group_name,
        &scope_and_user.scope,
        &[RoleType::Admin, RoleType::Curator, RoleType::Member],
        page_size,
        None,
    )?;
    Ok(HttpResponse::Ok().json(json!({ "members": members, "curators": curators, "group": group, "member_count": member_count, "next": page_size })))
}

pub fn views_app() -> impl HttpServiceFactory {
    web::scope("/views")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("/{group_name}/details").route(web::get().to(group_details)))
}

