use crate::api::error::ApiError;
use crate::api::models::DisplayGroupDetails;
use crate::api::models::GroupInfo;
use crate::db::operations;
use crate::db::operations::models::GroupUpdate;
use crate::db::operations::models::NewGroup;
use crate::db::operations::models::SortGroupsBy;
use crate::db::types::*;
use crate::db::Pool;
use crate::utils::valid_group_name;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use log::info;
use serde_derive::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct ListGroupsQuery {
    f: Option<String>,
    #[serde(default)]
    n: i64,
    #[serde(default = "default_groups_list_size")]
    s: i64,
    #[serde(default)]
    by: SortGroupsBy,
}

fn default_groups_list_size() -> i64 {
    20
}

#[guard(Authenticated)]
async fn get_group(pool: web::Data<Pool>, group_name: web::Path<String>) -> impl Responder {
    operations::groups::get_group(&pool, &group_name).map(|group| HttpResponse::Ok().json(group))
}

#[guard(Authenticated)]
async fn list_groups(pool: web::Data<Pool>, query: web::Query<ListGroupsQuery>) -> impl Responder {
    let query = query.into_inner();
    operations::groups::list_groups(&pool, query.f, query.by, query.s, query.n)
        .map(|groups| HttpResponse::Ok().json(groups))
        .map_err(ApiError::GenericBadRequest)
}

#[guard(Ndaed)]
async fn update_group(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    group_update: web::Json<GroupUpdate>,
    group_name: web::Path<String>,
) -> impl Responder {
    operations::groups::update_group(
        &pool,
        &scope_and_user,
        group_name.into_inner(),
        group_update.into_inner(),
    )
    .map(|_| HttpResponse::Created().finish())
    .map_err(ApiError::GenericBadRequest)
}

#[guard(Staff, Creator)]
async fn add_group(
    cis_client: web::Data<Arc<CisClient>>,
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    new_group: web::Json<NewGroup>,
) -> Result<HttpResponse, ApiError> {
    let new_group = new_group.into_inner();
    if !valid_group_name(&new_group.name) {
        return Err(ApiError::InvalidGroupName);
    }
    info!("trying to create new group: {}", new_group.name);
    operations::groups::add_new_group(&pool, &scope_and_user, new_group, Arc::clone(&*cis_client))
        .await?;
    Ok(HttpResponse::Created().finish())
}

async fn delete_group(
    cis_client: web::Data<Arc<CisClient>>,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> Result<HttpResponse, ApiError> {
    operations::groups::delete_group(&pool, &scope_and_user, &group_name, Arc::clone(&cis_client))
        .await?;
    Ok(HttpResponse::Created().finish())
}

#[guard(Authenticated)]
async fn group_details(
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> Result<HttpResponse, ApiError> {
    let host = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    let curator = operations::admins::is_admin(&pool, &scope_and_user, &group_name, &host);
    let page_size = 20;
    let member_count = match operations::members::member_count(&pool, &group_name) {
        Ok(member_count) => member_count,
        Err(e) => return Err(ApiError::GenericBadRequest(e)),
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
    let invitation_count = if curator {
        Some(operations::invitations::pending_invitations_count(
            &pool,
            &scope_and_user,
            &group_name,
            &host,
        )?)
    } else {
        None
    };
    let renewal_count = if curator {
        Some(operations::members::renewal_count(
            &pool,
            &group_name,
            None,
        )?)
    } else {
        None
    };
    let result = DisplayGroupDetails {
        curator,
        group: GroupInfo {
            name: group.group.name,
            description: group.group.description,
            typ: group.group.typ,
            expiration: if curator {
                group.group.group_expiration.or(Some(0))
            } else {
                None
            },
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
                .max_age(3600)
                .finish(),
        )
        .service(
            web::resource("")
                .route(web::post().to(add_group))
                .route(web::get().to(list_groups)),
        )
        .service(
            web::resource("/{group_name}")
                .route(web::get().to(get_group))
                .route(web::put().to(update_group))
                .route(web::delete().to(delete_group)),
        )
        .service(web::resource("/{group_name}/details").route(web::get().to(group_details)))
}
