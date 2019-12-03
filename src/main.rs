#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate failure_derive;

mod api;
mod cis;
mod db;
mod error;
mod healthz;
mod rules;
mod settings;
mod user;
mod utils;

use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use cis_client::CisClient;
use dino_park_gate::provider::Provider;
use dino_park_gate::scope::ScopeAndUserAuth;
use failure::Error;
use log::info;
use std::sync::Arc;

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("starting dino-park-whoami");

    let s = settings::Settings::new()?;
    let cis_client = Arc::new(CisClient::from_settings(&s.cis)?);

    let pool = db::db::establish_connection();
    let provider = Provider::from_issuer("https://auth.mozilla.auth0.com/")?;
    HttpServer::new(move || {
        let scope_middleware = ScopeAndUserAuth {
            checker: provider.clone(),
        };
        App::new()
            .data(Arc::clone(&cis_client))
            .data(pool.clone())
            .wrap(Logger::default().exclude("/healthz"))
            .service(healthz::healthz_app())
            .service(api::internal::internal_app())
            .service(
                web::scope("/groups/api/v1/")
                    .wrap(scope_middleware)
                    .service(api::groups::groups_app())
                    .service(api::members::members_app())
                    .service(api::views::views_app())
                    .service(api::current::current_app())
                    .service(api::invitations::invitations_app())
                    .service(api::terms::terms_app()),
            )
    })
    .bind("0.0.0.0:8085")?
    .run()
    .map_err(Into::into)
}
