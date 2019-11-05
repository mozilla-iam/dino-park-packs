use crate::db::db::establish_connection;
use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::operations::add_new_group;
use crate::db::schema;
use crate::db::schema::groups::dsl::*;
use crate::db::types::*;
use crate::user::User;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use cis_client::getby::GetBy;
use cis_client::AsyncCisClientTrait;
use cis_client::CisClient;
use diesel::prelude::*;
use dino_park_gate::scope::ScopeAndUser;
use futures::Future;
use serde_derive::Deserialize;
use std::convert::TryFrom;
use std::sync::Arc;
use uuid::Uuid;
use crate::cis::operations::add_group_to_profile;
use failure::format_err;

#[derive(Deserialize)]
struct NewGroup {
    description: String,
}

fn list_groups(_: HttpRequest, connection: web::Data<PgConnection>) -> HttpResponse {
    let results = groups
        .load::<Group>(&*connection)
        .expect("Error loading groups");

    println!("Displaying {} groups", results.len());
    for group in results {
        println!("{}", group.name);
    }
    HttpResponse::Ok().finish()
}

fn get_group(
    _: HttpRequest,
    connection: web::Data<PgConnection>,
    group_name: web::Path<String>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

fn update_group(
    _: HttpRequest,
    connection: web::Data<PgConnection>,
    group_name: web::Path<String>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}

fn add_group(
    _: HttpRequest,
    cis_client: web::Data<CisClient>,
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    new_group: web::Json<NewGroup>,
    group_name: web::Path<String>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let group_name = group_name.into_inner();
    let group_name_update = group_name.clone();
    let new_group = new_group.into_inner();
    let pool = pool.clone();
    let cis_client = cis_client.clone();
    cis_client
        .get_user_by(&scope_and_user.user_id, &GetBy::UserId, None)
        .and_then(|profile| (User::try_from(profile.clone()).map(|user| (user, profile))).map_err(Into::into))
        .and_then(|(user, profile)| {
            web::block(move || add_new_group(&pool, group_name, new_group.description, user))
                .map(move |_| profile)
                .map_err(|e| format_err!("{}", e))
        })
        .and_then(move |profile| add_group_to_profile(&cis_client, group_name_update, profile))
        .map(|_| HttpResponse::Ok().finish())
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
        .service(web::resource("").route(web::get().to(list_groups)))
        .service(
            web::resource("/{group_name}")
                .route(web::post().to_async(add_group))
                .route(web::get().to(get_group))
                .route(web::put().to(update_group)),
        )
}
