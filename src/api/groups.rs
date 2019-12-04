use crate::cis::operations::add_group_to_profile;
use crate::db::db::Pool;
use crate::db::operations;
use crate::db::types::*;
use crate::user::User;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use futures::future::IntoFuture;
use futures::Future;
use log::info;
use serde_derive::Deserialize;
use std::convert::TryFrom;
use std::sync::Arc;

#[derive(Deserialize)]
struct GroupUpdate {
    description: String,
}

#[derive(Deserialize)]
struct GroupCreate {
    name: String,
    typ: Option<GroupType>,
    description: String,
}

fn get_group(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
) -> impl Responder {
    operations::groups::get_group(&pool, &group_name)
        .map(|group| HttpResponse::Ok().json(group))
        .map_err(|_| HttpResponse::NotFound().finish())
}

fn update_group(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_update: web::Json<GroupUpdate>,
    group_name: web::Path<String>,
) -> impl Responder {
    operations::groups::update_group(
        &pool,
        group_name.into_inner(),
        Some(group_update.description.clone()),
        None,
        None,
        None,
    )
    .map(|_| HttpResponse::Created().finish())
    .map_err(|e| Error::from(e))
}

fn add_group(
    _: HttpRequest,
    cis_client: web::Data<Arc<CisClient>>,
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    new_group: web::Json<GroupCreate>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let new_group = new_group.into_inner();
    let group_name_update = new_group.name.clone();
    let pool = pool.clone();
    let cis_client = Arc::clone(&cis_client);
    let scope_and_user = scope_and_user.clone();
    info!("trying to create new group: {}", new_group.name);
    operations::users::user_profile_by_user_id(&pool, &scope_and_user.user_id)
        .and_then(|user_profile| {
            User::try_from(&user_profile.profile).map(|user| (user, user_profile))
        })
        .map_err(Into::into)
        .and_then(|(user, user_profile)| {
            operations::groups::add_new_group(
                &pool,
                &scope_and_user,
                new_group.name,
                new_group.description,
                user,
                new_group.typ.unwrap_or_else(|| GroupType::Closed),
                TrustType::Ndaed,
            )
            .and_then(|_| operations::users::update_user_cache(&pool, &user_profile.profile))
            .map(|_| user_profile)
        })
        .into_future()
        .and_then(move |user_profile| {
            add_group_to_profile(cis_client, group_name_update, user_profile.profile)
        })
        .map(|_| HttpResponse::Created().finish())
        .map_err(|e| Error::from(e))
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
                .route(web::put().to(update_group)),
        )
}
