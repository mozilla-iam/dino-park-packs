use crate::error::PacksError;
use crate::rules::error::RuleError;
use actix_web::error::ResponseError;
use actix_web::HttpResponse;
use dino_park_trust::GroupsTrustError;
use dino_park_trust::TrustError;
use log::warn;
use serde_json::json;
use serde_json::Value;
use std::fmt::Display;

#[derive(Fail, Debug)]
pub enum ApiError {
    #[fail(display = "A mulipart error occured.")]
    MultipartError,
    #[fail(display = "Bad API request.")]
    GenericBadRequest(failure::Error),
    #[fail(display = "Group names must ony containe alphanumeric charactars, -, and _")]
    InvalidGroupName,
    #[fail(display = "Operation Error: {}", _0)]
    PacksError(PacksError),
    #[fail(display = "Rule Error: {}", _0)]
    RuleError(RuleError),
    #[fail(display = "Scope Error: {}", _0)]
    ScopeError(TrustError),
    #[fail(display = "Groups scope Error: {}", _0)]
    GroupsScopeError(GroupsTrustError),
    #[fail(display = "Invalid query parameters.")]
    InvalidQuery,
}

fn to_json_error(e: &impl Display) -> Value {
    json!({ "error": e.to_string() })
}

impl From<TrustError> for ApiError {
    fn from(e: TrustError) -> Self {
        ApiError::ScopeError(e)
    }
}

impl From<GroupsTrustError> for ApiError {
    fn from(e: GroupsTrustError) -> Self {
        ApiError::GroupsScopeError(e)
    }
}

impl From<failure::Error> for ApiError {
    fn from(e: failure::Error) -> Self {
        let e = match e.downcast::<PacksError>() {
            Ok(e) => return ApiError::PacksError(e),
            Err(e) => e,
        };
        let e = match e.downcast::<RuleError>() {
            Ok(e) => return ApiError::RuleError(e),
            Err(e) => e,
        };
        ApiError::GenericBadRequest(e)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Self::GenericBadRequest(ref e) => {
                warn!("{}", e);
                HttpResponse::BadRequest().finish()
            }
            Self::PacksError(ref e) => HttpResponse::BadRequest().json(to_json_error(e)),
            Self::RuleError(ref e) => HttpResponse::Forbidden().json(to_json_error(e)),
            Self::ScopeError(ref e) => HttpResponse::Forbidden().json(to_json_error(e)),
            Self::GroupsScopeError(ref e) => HttpResponse::Forbidden().json(to_json_error(e)),
            Self::InvalidGroupName => HttpResponse::BadRequest().json(to_json_error(self)),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
