use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::operations::models::MembersQueryOptions;
use crate::db::operations::models::SortMembersBy;
use crate::db::types::RoleType;
use crate::db::Pool;
use crate::user::User;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
pub struct RenewMember {
    group_expiration: Option<i32>,
}

#[derive(Clone, Deserialize)]
pub enum MemberRoles {
    Any,
    Member,
    Curator,
}

impl MemberRoles {
    pub fn get_role_types(&self) -> Vec<RoleType> {
        match self {
            MemberRoles::Any => vec![RoleType::Admin, RoleType::Curator, RoleType::Member],
            MemberRoles::Member => vec![RoleType::Member],
            MemberRoles::Curator => vec![RoleType::Admin, RoleType::Curator],
        }
    }
}

#[derive(Deserialize)]
pub struct GetMembersQuery {
    q: Option<String>,
    r: Option<MemberRoles>,
    n: Option<i64>,
    s: Option<i64>,
    by: Option<SortMembersBy>,
}

impl From<GetMembersQuery> for MembersQueryOptions {
    fn from(q: GetMembersQuery) -> Self {
        let roles = q.r.unwrap_or_else(|| MemberRoles::Any).get_role_types();
        MembersQueryOptions {
            query: q.q,
            roles,
            limit: q.s.unwrap_or_else(|| 20),
            offset: q.n,
            order: q.by.unwrap_or_default(),
        }
    }
}

#[guard(Authenticated)]
async fn get_members(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    query: web::Query<GetMembersQuery>,
) -> impl Responder {
    match operations::members::scoped_members_and_host(
        &pool,
        &group_name,
        &scope_and_user.scope,
        query.into_inner().into(),
    ) {
        Ok(members) => Ok(HttpResponse::Ok().json(members)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Ndaed)]
async fn remove_member(
    pool: web::Data<Pool>,
    path: web::Path<(String, Uuid)>,
    scope_and_user: ScopeAndUser,
    cis_client: web::Data<Arc<CisClient>>,
) -> Result<HttpResponse, ApiError> {
    let (group_name, user_uuid) = path.into_inner();
    let user = User { user_uuid };
    let host = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    operations::members::remove(
        &pool,
        &scope_and_user,
        &group_name,
        &host,
        &user,
        Arc::clone(&*cis_client),
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}

#[guard(Ndaed)]
async fn renew_member(
    pool: web::Data<Pool>,
    path: web::Path<(String, Uuid)>,
    scope_and_user: ScopeAndUser,
    renew_member: web::Json<RenewMember>,
) -> impl Responder {
    let (group_name, user_uuid) = path.into_inner();
    let user = User { user_uuid };
    let host = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    match operations::members::renew(
        &pool,
        &scope_and_user,
        &group_name,
        &host,
        &user,
        renew_member.group_expiration,
    ) {
        Ok(_) => Ok(HttpResponse::Created().finish()),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

pub fn members_app() -> impl HttpServiceFactory {
    web::scope("/members")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .finish(),
        )
        .service(web::resource("/{group_name}").route(web::get().to(get_members)))
        .service(web::resource("/{group_name}/{user_uuid}").route(web::delete().to(remove_member)))
        .service(
            web::resource("/{group_name}/{user_uuid}/renew").route(web::post().to(renew_member)),
        )
}
