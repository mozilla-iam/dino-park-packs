use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use crate::user::User;
use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::web::Bytes;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::CisClient;
use cis_profile::schema::Profile;
use futures::StreamExt;
use futures::TryFutureExt;
use futures::TryStreamExt;
use serde_derive::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UpdatedProfiles {
    updated: usize,
}

async fn update_user(pool: web::Data<Pool>, profile: web::Json<Profile>) -> impl Responder {
    operations::users::update_user_cache(&pool, &profile).map(|_| HttpResponse::Ok().finish())
}

async fn delete_user(pool: web::Data<Pool>, user_uuid: web::Path<Uuid>) -> impl Responder {
    let user = User {
        user_uuid: user_uuid.into_inner(),
    };
    operations::users::delete_user(&pool, &user).map(|_| HttpResponse::Ok().finish())
}

async fn expire_all(
    pool: web::Data<Pool>,
    cis_client: web::Data<Arc<CisClient>>,
) -> Result<HttpResponse, ApiError> {
    operations::expirations::expire_invitations(&pool)?;
    operations::expirations::expire_memberships(&pool, Arc::clone(&*cis_client)).await?;
    Ok(HttpResponse::Ok().finish())
}

async fn bulk_update_users(
    pool: web::Data<Pool>,
    mut multipart: Multipart,
) -> Result<HttpResponse, ApiError> {
    let mut updated = 0;
    while let Some(item) = multipart.next().await {
        let field = item.map_err(|_| ApiError::MultipartError)?;
        let buf = field
            .try_fold(Vec::<u8>::new(), |mut acc: Vec<u8>, bytes: Bytes| {
                async move {
                    acc.extend(bytes.into_iter());
                    Ok(acc)
                }
            })
            .map_err(|_| ApiError::MultipartError)
            .await?;
        let profiles =
            serde_json::from_slice::<Vec<Profile>>(&buf).map_err(|_| ApiError::MultipartError)?;
        updated += operations::users::batch_update_user_cache(&pool, profiles)?;
    }
    Ok(HttpResponse::Ok().json(UpdatedProfiles { updated }))
}

pub fn internal_app() -> impl HttpServiceFactory {
    web::scope("/internal")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600)
                .finish(),
        )
        .data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("/update/bulk").route(web::post().to(bulk_update_users)))
        .service(web::resource("/update/user").route(web::post().to(update_user)))
        .service(web::resource("/delete/{user_uuid}").route(web::delete().to(delete_user)))
        .service(web::resource("/expire/all").route(web::post().to(expire_all)))
}
