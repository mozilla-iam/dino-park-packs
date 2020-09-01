use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::types::TrustType;
use crate::db::Pool;
use crate::user::User;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::AsyncCisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
pub struct ChangeTrust {
    trust: TrustType,
}

#[derive(Clone, Deserialize)]
pub struct AddMember {
    user_uuid: Uuid,
    group_expiration: Option<i32>,
}

#[derive(Deserialize)]
struct LimitOffsetQuery {
    #[serde(default)]
    n: i64,
    #[serde(default = "default_groups_list_size")]
    s: i64,
}

fn default_groups_list_size() -> i64 {
    20
}

#[guard(Staff, Admin, Medium)]
async fn add_member<T: AsyncCisClientTrait>(
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    add_member: web::Json<AddMember>,
    cis_client: web::Data<T>,
) -> Result<HttpResponse, ApiError> {
    let user_uuid = add_member.user_uuid;
    let host = operations::users::user_by_id(&pool.clone(), &scope_and_user.user_id)?;
    operations::members::add(
        &pool,
        &scope_and_user,
        &group_name,
        &host,
        &User { user_uuid },
        add_member.group_expiration,
        Arc::clone(&*cis_client),
    )
    .await?;
    Ok(HttpResponse::Ok().json(""))
}

#[guard(Staff, Admin, Medium)]
async fn all_raw_logs(pool: web::Data<Pool>, scope_and_user: ScopeAndUser) -> impl Responder {
    let user = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    match operations::logs::raw_logs(&pool, &scope_and_user, &user) {
        Ok(logs) => Ok(HttpResponse::Ok().json(logs)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Staff, Admin, Medium)]
async fn curator_emails(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    group_name: web::Path<String>,
) -> impl Responder {
    match operations::members::get_curator_emails(&pool, &scope_and_user, &group_name) {
        Ok(emails) => Ok(HttpResponse::Ok().json(emails)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Staff, Admin, Medium)]
async fn list_inactive_groups(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    query: web::Query<LimitOffsetQuery>,
) -> impl Responder {
    let query = query.into_inner();
    operations::groups::list_inactive_groups(&pool, &scope_and_user, query.s, query.n)
        .map(|groups| HttpResponse::Ok().json(groups))
        .map_err(ApiError::GenericBadRequest)
}

#[guard(Staff, Admin, Medium)]
async fn delete_inactive_group(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    group_name: web::Path<String>,
) -> impl Responder {
    operations::groups::delete_inactive_group(&pool, &scope_and_user, &group_name)
        .map(|_| HttpResponse::Ok().json(""))
        .map_err(ApiError::GenericBadRequest)
}

#[guard(Staff, Admin, Medium)]
async fn reserve_group(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    group_name: web::Path<String>,
) -> impl Responder {
    operations::groups::reserve_group(&pool, &scope_and_user, &group_name)
        .map(|_| HttpResponse::Ok().json(""))
        .map_err(ApiError::GenericBadRequest)
}

#[guard(Staff, Admin, Medium)]
async fn change_trust<T: AsyncCisClientTrait>(
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    trust_change: web::Json<ChangeTrust>,
    cis_client: web::Data<T>,
) -> Result<HttpResponse, ApiError> {
    operations::groups::update_group_trust(
        &pool,
        &scope_and_user,
        &group_name,
        &trust_change.trust,
        Arc::clone(&*cis_client),
    )
    .await?;
    Ok(HttpResponse::Ok().json(""))
}

pub fn sudo_app<T: AsyncCisClientTrait + 'static>() -> impl HttpServiceFactory {
    web::scope("/sudo")
        .service(web::resource("/groups/reserve/{group_name}").route(web::post().to(reserve_group)))
        .service(web::resource("/groups/inactive").route(web::get().to(list_inactive_groups)))
        .service(
            web::resource("/groups/inactive/{group_name}")
                .route(web::delete().to(delete_inactive_group)),
        )
        .service(
            web::resource("/trust/groups/{group_name}").route(web::put().to(change_trust::<T>)),
        )
        .service(web::resource("/member/{group_name}").route(web::post().to(add_member::<T>)))
        .service(web::resource("/curators/{group_name}").route(web::get().to(curator_emails)))
        .service(web::resource("/logs/all/raw").route(web::get().to(all_raw_logs)))
}
