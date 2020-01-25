#![warn(clippy::all)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate diesel_migrations;

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
use futures::TryFutureExt;
use log::info;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::Arc;

embed_migrations!();

fn map_io_err(e: impl Into<failure::Error>) -> Error {
    Error::new(ErrorKind::Other, e.into())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("starting dino-park-packs");

    let s = settings::Settings::new().map_err(map_io_err)?;
    let cis_client = Arc::new(CisClient::from_settings(&s.cis).map_err(map_io_err)?);

    let pool = db::establish_connection(&s.packs.postgres_url);
    embedded_migrations::run_with_output(&pool.get().map_err(map_io_err)?, &mut std::io::stdout())
        .map_err(map_io_err)?;
    let provider = Provider::from_issuer("https://auth.mozilla.auth0.com/")
        .map_err(map_io_err)
        .await?;
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
                    .service(api::current::current_app())
                    .service(api::invitations::invitations_app())
                    .service(api::terms::terms_app())
                    .service(api::users::users_app())
                    .service(api::admins::admins_app())
                    .service(api::sudo::sudo_app()),
            )
    })
    .bind("0.0.0.0:8085")?
    .run()
    .await
}
