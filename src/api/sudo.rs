use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use crate::user::User;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::AsyncCisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Deserialize)]
pub struct AddMember {
    user_uuid: Uuid,
    group_expiration: Option<i32>,
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
    Ok(HttpResponse::Ok().finish())
}

#[guard(Staff, Admin, Medium)]
async fn all_raw_logs(pool: web::Data<Pool>, scope_and_user: ScopeAndUser) -> impl Responder {
    let user = operations::users::user_by_id(&pool.clone(), &scope_and_user.user_id)?;
    match operations::logs::raw_logs(&pool, &scope_and_user, &user) {
        Ok(logs) => Ok(HttpResponse::Ok().json(logs)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

pub fn sudo_app<T: AsyncCisClientTrait + 'static>() -> impl HttpServiceFactory {
    web::scope("/sudo")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .finish(),
        )
        .service(web::resource("/member/{group_name}").route(web::post().to(add_member::<T>)))
        .service(web::resource("/logs/all/raw").route(web::get().to(all_raw_logs)))
}
