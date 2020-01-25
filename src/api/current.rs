use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use serde_derive::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Default)]
struct ForceLeave {
    force: Option<bool>,
}

async fn join(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    cis_client: web::Data<Arc<CisClient>>,
) -> Result<HttpResponse, ApiError> {
    let user = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    let user_profile = operations::users::user_profile_by_uuid(&pool, &user.user_uuid)?;
    operations::invitations::accept_invitation(
        &pool,
        &group_name,
        &user,
        Arc::clone(&*cis_client),
        user_profile.profile,
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}

async fn leave(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    force: web::Query<ForceLeave>,
    cis_client: web::Data<Arc<CisClient>>,
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

async fn reject(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    match operations::invitations::reject_invitation(&pool, &scope_and_user, &group_name) {
        Ok(_) => Ok(HttpResponse::Created().finish()),
        Err(e) => Err(ApiError::NotAcceptableError(e)),
    }
}

async fn invitations(pool: web::Data<Pool>, scope_and_user: ScopeAndUser) -> impl Responder {
    let user = operations::users::user_by_id(&pool.clone(), &scope_and_user.user_id)?;
    match operations::invitations::pending_invitations_for_user(&pool, &scope_and_user, &user) {
        Ok(invitations) => Ok(HttpResponse::Ok().json(invitations)),
        Err(e) => Err(ApiError::NotAcceptableError(e)),
    }
}

pub fn current_app() -> impl HttpServiceFactory {
    web::scope("/self")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .finish(),
        )
        .service(
            web::resource("/invitations/{group_name}")
                .route(web::delete().to(reject))
                .route(web::post().to(join)),
        )
        .service(web::resource("/invitations").route(web::get().to(invitations)))
        .service(web::resource("/{group_name}").route(web::delete().to(leave)))
}
