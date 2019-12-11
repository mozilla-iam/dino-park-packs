use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use crate::user::User;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpResponse;
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use futures::future::IntoFuture;
use futures::Future;
use serde_derive::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
pub struct AddAdmin {
    member_uuid: Uuid,
}

#[derive(Clone, Deserialize)]
pub struct DowngradeAdmin {
    group_expiration: Option<i32>,
}

fn add_admin(
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    add_admin: web::Json<AddAdmin>,
    cis_client: web::Data<Arc<CisClient>>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let pool_f = pool.clone();
    let user_uuid = add_admin.member_uuid;
    operations::users::user_by_id(&pool.clone(), &scope_and_user.user_id)
        .and_then(move |host| {
            operations::users::user_profile_by_uuid(&pool.clone(), &user_uuid)
                .map(|user_profile| (host, user_profile))
        })
        .into_future()
        .and_then(move |(host, user_profile)| {
            operations::admins::add_admin(
                &pool_f,
                &scope_and_user,
                &group_name,
                &host,
                &User { user_uuid },
                Arc::clone(&*cis_client),
                user_profile.profile,
            )
        })
        .map(|_| HttpResponse::Ok().finish())
        .map_err(Into::into)
}

pub fn downgrade(
    pool: web::Data<Pool>,
    path: web::Path<(String, Uuid)>,
    downgrade_admin: web::Json<DowngradeAdmin>,
    scope_and_user: ScopeAndUser,
) -> Result<HttpResponse, ApiError> {
    let (group_name, user_uuid) = path.into_inner();
    let host = operations::users::user_by_id(&pool, &scope_and_user.user_id)?;
    let user = User { user_uuid };
    log::info!("donwgrade");
    operations::admins::demote(
        &pool,
        &scope_and_user,
        &group_name,
        &host,
        &user,
        downgrade_admin.group_expiration,
    )
    .map(|_| HttpResponse::Created().finish())
    .map_err(ApiError::NotAcceptableError)
}

pub fn admins_app() -> impl HttpServiceFactory {
    web::scope("/curators")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .service(web::resource("/{group_name}").route(web::post().to_async(add_admin)))
        .service(
            web::resource("/{group_name}/{user_uuid}/downgrade").route(web::post().to(downgrade)),
        )
}
