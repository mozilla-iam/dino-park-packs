use crate::db::db::Pool;
use crate::db::operations;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use futures::future::Future;
use futures::future::IntoFuture;
use serde_derive::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Default)]
struct ForceLeave {
    force: Option<bool>,
}

fn join(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    cis_client: web::Data<Arc<CisClient>>,
) -> impl Future<Item = HttpResponse, Error = error::Error> {
    let pool_f = pool.clone();
    operations::users::user_by_id(&pool.clone(), &scope_and_user.user_id)
        .and_then(move |user| {
            operations::users::user_profile_by_uuid(&pool.clone(), &user.user_uuid)
                .map(|user_profile| (user, user_profile))
        })
        .into_future()
        .and_then(move |(user, user_profile)| {
            operations::invitations::accept_invitation(
                &pool_f,
                &group_name,
                &user,
                Arc::clone(&*cis_client),
                user_profile.profile,
            )
        })
        .map(|_| HttpResponse::Ok().finish())
        .map_err(|e| error::ErrorNotFound(e))
}

fn leave(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    force: web::Query<ForceLeave>,
    cis_client: web::Data<Arc<CisClient>>,
) -> impl Future<Item = HttpResponse, Error = error::Error> {
    let pool_f = pool.clone();
    operations::users::user_by_id(&pool.clone(), &scope_and_user.user_id)
        .and_then(move |user| {
            operations::users::user_profile_by_uuid(&pool.clone(), &user.user_uuid)
                .map(|user_profile| (user, user_profile))
        })
        .into_future()
        .and_then(move |(user, user_profile)| {
            operations::members::leave(
                &pool_f,
                &scope_and_user,
                &group_name,
                &user,
                force.force.unwrap_or_default(),
                Arc::clone(&*cis_client),
                user_profile.profile,
            )
        })
        .map(|_| HttpResponse::Ok().finish())
        .map_err(|e| error::ErrorNotFound(e))
}

fn invitations(pool: web::Data<Pool>, scope_and_user: ScopeAndUser) -> impl Responder {
    let user = operations::users::user_by_id(&pool.clone(), &scope_and_user.user_id)?;
    match operations::invitations::pending_invitations_for_user(&pool, &scope_and_user, &user) {
        Ok(invitations) => Ok(HttpResponse::Ok().json(invitations)),
        Err(e) => Err(error::ErrorNotFound(e)),
    }
}

pub fn current_app() -> impl HttpServiceFactory {
    web::scope("/self")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("/join/{group_name}").route(web::post().to_async(join)))
        .service(web::resource("/invitations").route(web::get().to(invitations)))
        .service(web::resource("/{group_name}").route(web::delete().to_async(leave)))
}
