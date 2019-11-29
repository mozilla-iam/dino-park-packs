use crate::db::db::Pool;
use crate::db::operations;
use crate::db::types::RoleType;
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
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use futures::future::IntoFuture;
use futures::Future;
use serde_derive::Deserialize;
use serde_humantime::De;
use std::sync::Arc;
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

fn get_members(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    query: web::Query<GetMembersQuery>,
) -> impl Responder {
    let page_size = query.size.unwrap_or_else(|| 20);
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

fn remove_member(
    _: HttpRequest,
    pool: web::Data<Pool>,
    path: web::Path<(String, Uuid)>,
    scope_and_user: ScopeAndUser,
    cis_client: web::Data<Arc<CisClient>>,
) -> impl Future<Item = HttpResponse, Error = error::Error> {
    // TODO: rules
    let pool_f = pool.clone();
    let (group_name, user_uuid) = path.into_inner();
    let user = User {
        user_uuid: user_uuid,
    };
    operations::users::user_by_id(&pool, &scope_and_user.user_id)
        .and_then(|host| {
            operations::users::user_profile_by_uuid(&pool.clone(), &user.user_uuid)
                .map(|user_profile| (host, user, user_profile))
        })
        .into_future()
        .and_then(move |(host, user, user_profile)| {
            operations::members::remove(
                &pool_f,
                &scope_and_user,
                &group_name,
                &host,
                &user,
                Arc::clone(&*cis_client),
                user_profile.profile,
            )
        })
        .map(|_| HttpResponse::Ok().finish())
        .map_err(|e| error::ErrorNotFound(e))
}

pub fn pending(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
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
        .service(web::resource("/{group_name}/invite").route(web::post().to(invite_member)))
        .service(
            web::resource("/{group_name}/{user_uuid}").route(web::delete().to_async(remove_member)),
        )
}
