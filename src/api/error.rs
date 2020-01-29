use actix_web::error::ResponseError;
use actix_web::HttpResponse;
use log::warn;
use serde_json::json;
use serde_json::Value;

#[derive(Fail, Debug)]
pub enum ApiError {
    #[fail(display = "A mulipart error occured.")]
    MultipartError,
    #[fail(display = "This API request is not acceptable.")]
    NotAcceptableError(failure::Error),
    #[fail(display = "Group names must ony containe alphanumeric charactars, -, and _")]
    InvalidGroupName,
}

fn to_json_error(e: &ApiError) -> Value {
    json!({ "error": e.to_string() })
}

impl From<failure::Error> for ApiError {
    fn from(e: failure::Error) -> Self {
        ApiError::NotAcceptableError(e)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Self::NotAcceptableError(ref e) => {
                warn!("{}", e);
                HttpResponse::NotAcceptable().finish()
            }
            Self::InvalidGroupName => HttpResponse::BadRequest().json(to_json_error(self)),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
