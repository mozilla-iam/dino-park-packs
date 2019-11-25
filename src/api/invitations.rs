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
pub struct Invitation {
    user_uuid: Uuid,
    invitation_expiration: De<Option<Duration>>,
    group_expiration: De<Option<Duration>>,
}

#[derive(Deserialize)]
pub struct GetMembersQuery {
    next: Option<i64>,
    size: Option<i64>,
}

fn invite_member(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    invitation: web::Json<Invitation>,
) -> impl Responder {
    let invitation = invitation.into_inner();
    let invitation_expiration = match invitation
        .invitation_expiration
        .into_inner()
        .map(to_expiration_ts)
    {
        Some(Err(e)) => return Err(error::ErrorBadRequest(e)),
        Some(Ok(ts)) => Some(ts),
        None => None,
    };
    let group_expiration = match invitation
        .group_expiration
        .into_inner()
        .map(to_expiration_ts)
    {
        Some(Err(e)) => return Err(error::ErrorBadRequest(e)),
        Some(Ok(ts)) => Some(ts),
        None => None,
    };
    let member = User {
        user_uuid: invitation.user_uuid,
    };
    let host = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    match operations::invitations::invite_member(
        &pool,
        &scope_and_user,
        &group_name,
        host,
        member,
        invitation_expiration,
        group_expiration,
    ) {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => Err(error::ErrorNotFound(e)),
    }
}

pub fn pending(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    let host = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    match operations::invitations::pending_invitations(&pool, &scope_and_user, &group_name, &host) {
        Ok(invitations) => Ok(HttpResponse::Ok().json(invitations)),
        Err(e) => Err(error::ErrorNotFound(e)),
    }
}

pub fn invitations_app() -> impl HttpServiceFactory {
    web::scope("/invitations")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(
            web::resource("/{group_name}")
                .route(web::get().to(pending))
                .route(web::post().to(invite_member)),
        )
}
