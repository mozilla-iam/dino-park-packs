use crate::db::establish_connection;
use crate::groups::group::*;
use crate::groups::operations::add_new_group;
use crate::groups::operations::add_user_to_group;
use crate::groups::schema;
use crate::groups::schema::groups::dsl::*;
use crate::types::*;
use crate::user::User;
use actix_cors::Cors;
use actix_web::dev::HttpServiceFactory;
use actix_web::http;
use actix_web::web;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use diesel::prelude::*;
use uuid::Uuid;

fn some(_: HttpRequest, connection: web::Data<PgConnection>) -> HttpResponse {
    let results = groups
        .load::<Group>(&*connection)
        .expect("Error loading groups");

    println!("Displaying {} groups", results.len());
    for group in results {
        println!("{}", group.name);
    }
    HttpResponse::Ok().finish()
}

fn add_some(
    _: HttpRequest,
    connection: web::Data<PgConnection>,
    group_name: web::Path<String>,
) -> HttpResponse {
    add_new_group(
        &*connection,
        group_name.clone(),
        String::from("something"),
        User::default(),
    );

    HttpResponse::Ok().finish()
}

fn add_some_user(
    _: HttpRequest,
    connection: web::Data<PgConnection>,
    group_user: web::Path<(String, Uuid)>,
) -> HttpResponse {
    let (group_name, user_uuid) = group_user.into_inner();
    add_user_to_group(
        &*connection,
        group_name,
        User::default(),
        User { user_uuid },
    );
    HttpResponse::Ok().finish()
}

pub fn some_app() -> impl HttpServiceFactory {
    let connection = establish_connection();
    web::scope("/some")
        .wrap(
            Cors::new()
                .allowed_methods(vec!["GET", "HEAD"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .data(connection)
        .service(web::resource("").to(some))
        .service(web::resource("add/group/{group_name}").to(add_some))
        .service(web::resource("add/member/{group_name}/{user_uuid}").to(add_some_user))
}
