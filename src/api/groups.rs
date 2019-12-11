use crate::api::error::ApiError;
use crate::api::models::DisplayGroupDetails;
use crate::api::models::GroupInfo;
use crate::db::operations;
use crate::db::operations::models::NewGroup;
use crate::db::types::*;
use crate::db::Pool;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use futures::Future;
use log::info;
use serde_derive::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct GroupUpdate {
    description: Option<String>,
    typ: Option<GroupType>,
    group_expiration: Option<i32>,
}

#[derive(Deserialize)]
struct GroupCreate {
    name: String,
    typ: Option<GroupType>,
    description: String,
    group_expiration: Option<i32>,
}

fn get_group(pool: web::Data<Pool>, group_name: web::Path<String>) -> impl Responder {
    operations::groups::get_group(&pool, &group_name)
        .map(|group| HttpResponse::Ok().json(group))
        .map_err(ApiError::NotAcceptableError)
}

fn update_group(
    pool: web::Data<Pool>,
    group_update: web::Json<GroupUpdate>,
    group_name: web::Path<String>,
) -> impl Responder {
    let group_update = group_update.into_inner();
    operations::groups::update_group(
        &pool,
        group_name.into_inner(),
        group_update.description,
        None,
        group_update.typ,
        group_update
            .group_expiration
            .map(|i| if i < 1 { None } else { Some(i) }),
    )
    .map(|_| HttpResponse::Created().finish())
    .map_err(ApiError::NotAcceptableError)
}

fn add_group(
    cis_client: web::Data<Arc<CisClient>>,
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    new_group: web::Json<GroupCreate>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let new_group = new_group.into_inner();
    let pool = pool.clone();
    let cis_client = Arc::clone(&cis_client);
    let scope_and_user = scope_and_user.clone();
    info!("trying to create new group: {}", new_group.name);
    let new_group = NewGroup {
        name: new_group.name,
        description: new_group.description,
        typ: new_group.typ.unwrap_or_else(|| GroupType::Closed),
        trust: TrustType::Ndaed,
        expiration: new_group
            .group_expiration
            .and_then(|i| if i < 1 { None } else { Some(i) }),
    };
    operations::groups::add_new_group(&pool, &scope_and_user, new_group, cis_client)
        .map(|_| HttpResponse::Created().finish())
        .map_err(ApiError::NotAcceptableError)
}

fn delete_group(
    cis_client: web::Data<Arc<CisClient>>,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    operations::groups::delete_group(&pool, &scope_and_user, &group_name, Arc::clone(&cis_client))
        .map(|_| HttpResponse::Created().finish())
        .map_err(ApiError::NotAcceptableError)
}

fn group_details(
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> Result<HttpResponse, ApiError> {
    let host = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    let page_size = 20;
    let member_count = match operations::members::member_count(&pool, &group_name) {
        Ok(member_count) => member_count,
        Err(e) => return Err(ApiError::NotAcceptableError(e)),
    };
    let group = operations::groups::get_group_with_terms_flag(&pool, &group_name)?;
    let members = operations::members::scoped_members_and_host(
        &pool,
        &group_name,
        &scope_and_user.scope,
        None,
        &[RoleType::Admin, RoleType::Curator, RoleType::Member],
        page_size,
        None,
    )?;
    let invitation_count = operations::invitations::pending_invitations_count(
        &pool,
        &scope_and_user,
        &group_name,
        &host,
    )?;
    let renewal_count = operations::members::renewal_count(&pool, &group_name, None)?;
    let result = DisplayGroupDetails {
        group: GroupInfo {
            name: group.group.name,
            description: group.group.description,
            typ: group.group.typ,
            terms: group.terms,
        },
        members,
        member_count,
        invitation_count,
        renewal_count,
    };
    Ok(HttpResponse::Ok().json(result))
}

pub fn groups_app() -> impl HttpServiceFactory {
    web::scope("/groups")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("").route(web::post().to_async(add_group)))
        .service(
            web::resource("/{group_name}")
                .route(web::get().to(get_group))
                .route(web::put().to(update_group))
                .route(web::delete().to_async(delete_group)),
        )
        .service(web::resource("/{group_name}/details").route(web::get().to(group_details)))
}
