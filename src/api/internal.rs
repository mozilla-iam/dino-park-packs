use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use crate::user::User;
use actix_multipart::Multipart;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::web::Bytes;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::AsyncCisClientTrait;
use cis_profile::schema::Profile;
use futures::StreamExt;
use futures::TryFutureExt;
use futures::TryStreamExt;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UpdatedProfiles {
    updated: usize,
}

async fn update_user(pool: web::Data<Pool>, profile: web::Json<Profile>) -> impl Responder {
    operations::users::update_user_cache(&pool, &profile).map(|_| HttpResponse::Ok().json(""))
}

async fn delete_user(pool: web::Data<Pool>, user_uuid: web::Path<Uuid>) -> impl Responder {
    let user = User {
        user_uuid: user_uuid.into_inner(),
    };
    operations::users::delete_user(&pool, &user).map(|_| HttpResponse::Ok().json(""))
}

async fn expire_all<T: AsyncCisClientTrait>(
    pool: web::Data<Pool>,
    cis_client: web::Data<T>,
) -> Result<HttpResponse, ApiError> {
    operations::expirations::expire_invitations(&pool)?;
    operations::expirations::expire_memberships(&pool, Arc::clone(&*cis_client)).await?;
    Ok(HttpResponse::Ok().json(""))
}

async fn expiration_notifications(pool: web::Data<Pool>) -> Result<HttpResponse, ApiError> {
    operations::expirations::expiration_notification(&pool, true)?;
    operations::expirations::expiration_notification(&pool, false)?;
    Ok(HttpResponse::Ok().json(""))
}

async fn bulk_update_users(
    pool: web::Data<Pool>,
    mut multipart: Multipart,
) -> Result<HttpResponse, ApiError> {
    let mut updated = 0;
    while let Some(item) = multipart.next().await {
        let field = item.map_err(|_| ApiError::MultipartError)?;
        let buf = field
            .try_fold(
                Vec::<u8>::new(),
                |mut acc: Vec<u8>, bytes: Bytes| async move {
                    acc.extend(bytes.into_iter());
                    Ok(acc)
                },
            )
            .map_err(|_| ApiError::MultipartError)
            .await?;
        let profiles =
            serde_json::from_slice::<Vec<Profile>>(&buf).map_err(|_| ApiError::MultipartError)?;
        updated += operations::users::batch_update_user_cache(&pool, profiles)?;
    }
    Ok(HttpResponse::Ok().json(UpdatedProfiles { updated }))
}

pub fn internal_app<T: AsyncCisClientTrait + 'static>() -> impl HttpServiceFactory {
    web::scope("/internal")
        .app_data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("/update/bulk").route(web::post().to(bulk_update_users)))
        .service(web::resource("/update/user").route(web::post().to(update_user)))
        .service(web::resource("/delete/{user_uuid}").route(web::delete().to(delete_user)))
        .service(web::resource("/expire/all").route(web::post().to(expire_all::<T>)))
        .service(web::resource("/expire/notify").route(web::post().to(expiration_notifications)))
}
