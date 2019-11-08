#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate failure_derive;

mod api;
mod cis;
mod db;
mod healthz;
mod settings;
mod user;
mod utils;
mod error;

use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use cis_client::CisClient;
use dino_park_gate::provider::Provider;
use dino_park_gate::scope::ScopeAndUserAuth;
use failure::Error;
use log::info;

fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("starting dino-park-whoami");

    let s = settings::Settings::new()?;
    let cis_client = CisClient::from_settings(&s.cis)?;

    let pool = db::db::establish_connection();
    let provider = Provider::from_issuer("https://auth.mozilla.auth0.com/")?;
    HttpServer::new(move || {
        let scope_middleware = ScopeAndUserAuth {
            checker: provider.clone(),
        };
        App::new()
            .data(cis_client.clone())
            .data(pool.clone())
            .wrap(Logger::default().exclude("/healthz"))
            .service(healthz::healthz_app())
            .service(
                web::scope("/api/v1/")
                    .wrap(scope_middleware)
                    .service(api::groups::groups_app())
                    .service(api::members::members_app())
                    .service(api::views::views_app()),
            )
    })
    .bind("0.0.0.0:8085")?
    .run()
    .map_err(Into::into)
}
