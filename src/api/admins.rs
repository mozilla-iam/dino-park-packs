use crate::db::db::Pool;
use crate::db::operations;
use crate::user::User;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
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

fn add_admin(
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
    add_admin: web::Json<AddAdmin>,
    cis_client: web::Data<Arc<CisClient>>,
) -> impl Future<Item = HttpResponse, Error = error::Error> {
    let pool_f = pool.clone();
    let user_uuid = add_admin.member_uuid.clone();
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
        .map_err(|e| error::ErrorNotFound(e))
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
        .service(web::resource("/{group_name}")
                .route(web::post().to_async(add_admin))
        //.route(web::get().to(get_admins)))
        //.service(
        //    web::resource("/{group_name}/{user_uuid}")
        //        .route(web::delete().to_async(remove_admin)),
        )
}
