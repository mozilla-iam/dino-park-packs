use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::types::RoleType;
use crate::db::types::TrustType;
use crate::db::Pool;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use dino_park_gate::scope::ScopeAndUser;
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchUsersQuery {
    q: String,
    t: Option<TrustType>,
    g: String,
    #[serde(default)]
    c: bool,
    #[serde(default)]
    a: bool,
}

#[derive(Deserialize)]
struct SearchAllUsersQuery {
    q: String,
    t: Option<TrustType>,
    l: i64,
}

#[guard(Ndaed)]
async fn search_users(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    query: web::Query<SearchUsersQuery>,
) -> impl Responder {
    let query = query.into_inner();
    let curators = query.c;
    let mut users = if curators {
        operations::users::search_admins(&pool, scope_and_user, query.g, &query.q)
    } else {
        operations::users::search_users(&pool, scope_and_user, query.g, query.t, &query.q)
    };
    if !query.a {
        users = users.map(|users| {
            users
                .into_iter()
                .filter(|u| {
                    if curators {
                        u.role.is_none() || u.role == Some(RoleType::Member)
                    } else {
                        !u.invited && u.role.is_none()
                    }
                })
                .collect()
        })
    }
    match users {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Staff, Admin)]
async fn search_all_users(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    query: web::Query<SearchAllUsersQuery>,
) -> impl Responder {
    let query = query.into_inner();
    let users =
        operations::users::search_all_users(&pool, scope_and_user, query.t, &query.q, query.l);
    match users {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

pub fn users_app() -> impl HttpServiceFactory {
    web::scope("/users")
        .service(web::resource("").route(web::get().to(search_users)))
        .service(web::resource("/all").route(web::get().to(search_all_users)))
}
