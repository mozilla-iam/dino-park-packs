use actix_web::error::ResponseError;
use actix_web::HttpResponse;
use log::warn;

#[derive(Fail, Debug)]
pub enum ApiError {
    #[fail(display = "A mulipart error occured.")]
    MultipartError,
    #[fail(display = "This API request is not acceptable.")]
    NotAcceptableError(failure::Error),
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
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
