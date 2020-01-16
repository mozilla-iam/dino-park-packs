use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_multipart::MultipartError;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::http;
use actix_web::web;
use actix_web::web::Bytes;
use actix_web::HttpResponse;
use actix_web::Responder;
use cis_client::CisClient;
use cis_profile::schema::Profile;
use futures::future;
use futures::future::IntoFuture;
use futures::Future;
use futures::Stream;
use log::error;
use serde_derive::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct UpdatedProfiles {
    updated: usize,
}

fn update_user(pool: web::Data<Pool>, profile: web::Json<Profile>) -> impl Responder {
    operations::users::update_user_cache(&pool, &profile).map(|_| HttpResponse::Ok().finish())
}

fn expire_all(
    pool: web::Data<Pool>,
    cis_client: web::Data<Arc<CisClient>>,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    let pool_f = pool.clone();
    let cis_client = Arc::clone(&cis_client);
    operations::expirations::expire_invitations(&pool)
        .into_future()
        .and_then(move |_| operations::expirations::expire_memberships(&pool_f, cis_client))
        .map_err(Into::into)
        .map(|_| HttpResponse::Ok().finish())
}

fn bulk_update_users(
    pool: web::Data<Pool>,
    multipart: Multipart,
) -> impl Future<Item = HttpResponse, Error = ApiError> {
    multipart
        .map(move |field| {
            let pool = pool.clone();
            field
                .fold(Vec::<u8>::new(), |mut acc: Vec<u8>, bytes: Bytes| {
                    acc.extend(bytes.into_iter());
                    future::result(Ok(acc).map_err(|e| {
                        error!("failed multipart, {:?}", e);
                        MultipartError::Payload(error::PayloadError::Io(e))
                    }))
                })
                .map_err(|e| {
                    error!("failed multipart, {:?}", e);
                    ApiError::MultipartError
                })
                .and_then(move |buf: Vec<u8>| {
                    serde_json::from_slice::<Vec<Profile>>(&buf)
                        .map_err(Into::into)
                        .and_then(|profiles| {
                            operations::users::batch_update_user_cache(&pool, profiles)
                        })
                        .map_err(Into::into)
                        .into_future()
                })
                .into_stream()
        })
        .map_err(|_| ApiError::MultipartError)
        .flatten()
        .collect()
        .map(|mut v| v.pop().unwrap_or_default())
        .map_err(Into::into)
        .map(|updated| HttpResponse::Ok().json(UpdatedProfiles { updated }))
}

pub fn internal_app() -> impl HttpServiceFactory {
    web::scope("/internal")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "PUT", "POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("/update/bulk").route(web::post().to_async(bulk_update_users)))
        .service(web::resource("/update/user").route(web::post().to(update_user)))
        .service(web::resource("/expire/all").route(web::post().to_async(expire_all)))
}
