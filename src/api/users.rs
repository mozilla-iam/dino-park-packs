use crate::api::error::ApiError;
use crate::db::operations;
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
    g: Option<String>,
    #[serde(default)]
    c: bool,
}

#[guard(Ndaed)]
async fn search_users(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    query: web::Query<SearchUsersQuery>,
) -> impl Responder {
    let query = query.into_inner();
    let users = if query.c {
        match query.g {
            Some(group_name) => {
                operations::users::search_admins(&pool, scope_and_user, group_name, &query.q)
            }
            _ => return Err(ApiError::InvalidQuery),
        }
    } else {
        operations::users::search_users(&pool, scope_and_user, query.g, query.t, &query.q)
    };
    match users {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

pub fn users_app() -> impl HttpServiceFactory {
    web::scope("/users").service(web::resource("").route(web::get().to(search_users)))
}
