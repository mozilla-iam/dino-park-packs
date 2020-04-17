use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::AsyncCisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Default)]
struct ForceLeave {
    force: Option<bool>,
}

#[guard(Authenticated)]
async fn join<T: AsyncCisClientTrait>(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    cis_client: web::Data<T>,
) -> Result<HttpResponse, ApiError> {
    let user = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    operations::invitations::accept_invitation(
        &pool,
        &scope_and_user,
        &group_name,
        &user,
        Arc::clone(&*cis_client),
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}

#[guard(Authenticated)]
async fn leave<T: AsyncCisClientTrait>(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    force: web::Query<ForceLeave>,
    cis_client: web::Data<T>,
) -> Result<HttpResponse, ApiError> {
    operations::members::leave(
        &pool,
        &scope_and_user,
        &group_name,
        force.force.unwrap_or_default(),
        Arc::clone(&*cis_client),
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}

#[guard(Ndaed)]
async fn request(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    match operations::requests::request_membership(&pool, &scope_and_user, &group_name, None) {
        Ok(_) => Ok(HttpResponse::Created().finish()),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Ndaed)]
async fn cancel_request(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    match operations::requests::cancel_request(&pool, &scope_and_user, &group_name) {
        Ok(_) => Ok(HttpResponse::Created().finish()),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Authenticated)]
async fn reject_invitation(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    match operations::invitations::reject_invitation(&pool, &scope_and_user, &group_name) {
        Ok(_) => Ok(HttpResponse::Created().finish()),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Authenticated)]
async fn invitations(pool: web::Data<Pool>, scope_and_user: ScopeAndUser) -> impl Responder {
    match operations::invitations::pending_invitations_for_user(&pool, &scope_and_user) {
        Ok(invitations) => Ok(HttpResponse::Ok().json(invitations)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Authenticated)]
async fn requests(pool: web::Data<Pool>, scope_and_user: ScopeAndUser) -> impl Responder {
    match operations::requests::pending_requests_for_user(&pool, &scope_and_user) {
        Ok(requests) => Ok(HttpResponse::Ok().json(requests)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

pub fn current_app<T: AsyncCisClientTrait + 'static>() -> impl HttpServiceFactory {
    web::scope("/self")
        .service(
            web::resource("/invitations/{group_name}")
                .route(web::delete().to(reject_invitation))
                .route(web::post().to(join::<T>)),
        )
        .service(web::resource("/invitations").route(web::get().to(invitations)))
        .service(
            web::resource("/requests/{group_name}")
                .route(web::post().to(request))
                .route(web::delete().to(cancel_request)),
        )
        .service(web::resource("/requests").route(web::get().to(requests)))
        .service(web::resource("/{group_name}").route(web::delete().to(leave::<T>)))
}
