use crate::api::error::ApiError;
use crate::db::types::TrustType;
use crate::db::Pool;
use crate::import::ops::*;
use crate::import::tsv::*;
use actix_http::http::header::DispositionParam;
use actix_multipart::Multipart;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::web::Bytes;
use actix_web::HttpResponse;
use cis_client::AsyncCisClientTrait;
use csv::ReaderBuilder;
use futures::StreamExt;
use futures::TryFutureExt;
use futures::TryStreamExt;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Default)]
struct GroupImportOption {
    pub group: Option<MozilliansGroup>,
    pub memberships: Option<Vec<MozilliansGroupMembership>>,
    pub curators: Option<Vec<MozilliansGroupCurator>>,
}

#[derive(Debug, Deserialize, Default)]
struct GroupImportQuery {
    trust: Option<TrustType>,
}

async fn legacy_user_data_import(
    mut multipart: Multipart,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ApiError> {
    let mut legacy_user_data = None;
    while let Some(Ok(field)) = multipart.next().await {
        let typ = if let Some(cd) = field.content_disposition() {
            match cd.parameters.get(0) {
                Some(DispositionParam::Name(n)) => n.clone(),
                _ => return Err(ApiError::MultipartError),
            }
        } else {
            return Err(ApiError::MultipartError);
        };
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
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(buf.as_slice());
        match typ.as_str() {
            "data" => {
                let d = rdr
                    .deserialize()
                    .collect::<Result<Vec<LegacyUserDataRaw>, _>>()
                    .map_err(|_| ApiError::MultipartError)?;
                legacy_user_data = Some(d);
            }
            _ => return Err(ApiError::MultipartError),
        }
    }
    match legacy_user_data {
        Some(legacy_user_data) => import_legacy_user_data(&pool, legacy_user_data)?,
        _ => return Err(ApiError::MultipartError),
    }
    Ok(HttpResponse::Ok().finish())
}

async fn full_group_import<T: AsyncCisClientTrait>(
    mut multipart: Multipart,
    pool: web::Data<Pool>,
    cis_client: web::Data<T>,
    query: web::Query<GroupImportQuery>,
) -> Result<HttpResponse, ApiError> {
    let trust = query.into_inner().trust.unwrap_or(TrustType::Ndaed);
    let mut group_import = GroupImportOption {
        ..Default::default()
    };
    while let Some(Ok(field)) = multipart.next().await {
        let typ = if let Some(cd) = field.content_disposition() {
            match cd.parameters.get(0) {
                Some(DispositionParam::Name(n)) => n.clone(),
                _ => return Err(ApiError::MultipartError),
            }
        } else {
            return Err(ApiError::MultipartError);
        };
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
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(buf.as_slice());
        match typ.as_str() {
            "group" => {
                let mut g = rdr
                    .deserialize()
                    .collect::<Result<Vec<MozilliansGroup>, _>>()
                    .map_err(|_| ApiError::MultipartError)?;
                group_import.group = g.pop();
            }
            "memberships" => {
                let m = rdr
                    .deserialize()
                    .collect::<Result<Vec<MozilliansGroupMembership>, _>>()
                    .map_err(|_| ApiError::MultipartError)?;
                group_import.memberships = Some(m);
            }
            "curators" => {
                let c = rdr
                    .deserialize()
                    .collect::<Result<Vec<MozilliansGroupCurator>, _>>()
                    .map_err(|_| ApiError::MultipartError)?;
                group_import.curators = Some(c);
            }
            _ => return Err(ApiError::MultipartError),
        }
    }
    match group_import {
        GroupImportOption {
            group: Some(group),
            curators: Some(curators),
            memberships: Some(memberships),
        } => {
            let group_import = GroupImport {
                group,
                curators,
                memberships,
                trust,
            };
            import(&pool, group_import, Arc::clone(&*cis_client)).await?
        }
        _ => return Err(ApiError::MultipartError),
    }
    Ok(HttpResponse::Ok().finish())
}

pub fn import_app<T: AsyncCisClientTrait + 'static>() -> impl HttpServiceFactory {
    web::scope("/import")
        .app_data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("/group/full").route(web::post().to(full_group_import::<T>)))
        .service(web::resource("/legacy/userdata").route(web::post().to(legacy_user_data_import)))
}
