use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use crate::user::User;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use dino_park_gate::scope::ScopeAndUser;
use uuid::Uuid;

#[guard(Ndaed, None, Medium)]
async fn reject(
    _: HttpRequest,
    pool: web::Data<Pool>,
    path: web::Path<(String, Uuid)>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    let (group_name, user_uuid) = path.into_inner();
    let member = User { user_uuid };
    match operations::requests::reject_request(&pool, &scope_and_user, &group_name, &member) {
        Ok(_) => Ok(HttpResponse::Created().json("")),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

#[guard(Ndaed)]
async fn pending(
    _: HttpRequest,
    pool: web::Data<Pool>,
    group_name: web::Path<String>,
    scope_and_user: ScopeAndUser,
) -> impl Responder {
    match operations::requests::pending_requests(&pool, &scope_and_user, &group_name) {
        Ok(requests) => Ok(HttpResponse::Ok().json(requests)),
        Err(e) => Err(ApiError::GenericBadRequest(e)),
    }
}

pub fn requests_app() -> impl HttpServiceFactory {
    web::scope("/requests")
        .service(web::resource("/{group_name}/{user_uuid}").route(web::delete().to(reject)))
        .service(web::resource("/{group_name}").route(web::get().to(pending)))
}
