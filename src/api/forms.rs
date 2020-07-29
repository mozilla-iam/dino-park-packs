use crate::api::error::ApiError;
use crate::db::operations;
use crate::db::Pool;
use crate::mail::manager::send_email_raw;
use crate::mail::Email;
use crate::mail::Message;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use actix_web::HttpResponse;
use cis_client::AsyncCisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BccEmail {
    pub bcc: String,
    pub body: String,
    pub subject: String,
}

#[derive(Deserialize)]
pub struct GroupEmail {
    pub body: String,
    pub subject: String,
    pub group_name: String,
}

#[guard(Staff, Admin, Medium)]
async fn email_bcc(bcc_email: web::Form<BccEmail>) -> Result<HttpResponse, ApiError> {
    let bcc_email = bcc_email.into_inner();
    let email = Email {
        message: Message {
            body: bcc_email.body,
            subject: bcc_email.subject,
        },
        bcc: Some(
            bcc_email
                .bcc
                .split(',')
                .map(|s| String::from(s.trim()))
                .collect(),
        ),
        ..Default::default()
    };
    send_email_raw(email);
    Ok(HttpResponse::Ok().body("ok"))
}

#[guard(Staff, Admin, Medium)]
async fn email_group(
    pool: web::Data<Pool>,
    scope_and_user: ScopeAndUser,
    group_email: web::Form<GroupEmail>,
) -> Result<HttpResponse, ApiError> {
    let GroupEmail {
        body,
        subject,
        group_name,
    } = group_email.into_inner();

    let bcc = Some(operations::members::get_member_emails(
        &pool,
        &scope_and_user,
        &group_name,
    )?);
    let message = Message { body, subject };
    let email = Email {
        bcc,
        message,
        ..Default::default()
    };
    send_email_raw(email);
    Ok(HttpResponse::Ok().body("ok"))
}

#[guard(Staff, Admin, Medium)]
async fn form() -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../static/form.html")))
}

pub fn forms_app<T: AsyncCisClientTrait + 'static>() -> impl HttpServiceFactory {
    web::scope("/forms")
        .service(web::resource("/email/").route(web::get().to(form)))
        .service(web::resource("/email/bcc").route(web::post().to(email_bcc)))
        .service(web::resource("/email/group").route(web::post().to(email_group)))
}
