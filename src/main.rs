#![warn(clippy::all)]
#[macro_use]
extern crate diesel_migrations;

use dino_park_packs::*;

use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use cis_client::CisClient;
use dino_park_gate::provider::Provider;
use dino_park_gate::scope::ScopeAndUserAuth;
use log::info;
use log::debug;
use std::io::Error;
use std::io::ErrorKind;

embed_migrations!();

fn map_io_err(e: impl Into<failure::Error>) -> Error {
    Error::new(ErrorKind::Other, e.into())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("starting dino-park-packs");
    debug!("DEBUG logging enabled");

    let s = settings::Settings::new().map_err(map_io_err)?;
    let cis_client = CisClient::from_settings(&s.cis).await.map_err(map_io_err)?;

    let pool = db::establish_connection(&s.packs.postgres_url);
    embedded_migrations::run_with_output(&pool.get().map_err(map_io_err)?, &mut std::io::stdout())
        .map_err(map_io_err)?;

    let provider = Provider::from_issuer(&s.auth).await.map_err(map_io_err)?;
    HttpServer::new(move || {
        let scope_middleware = ScopeAndUserAuth::new(provider.clone());
        App::new()
            .data(cis_client.clone())
            .data(pool.clone())
            .wrap(Logger::default().exclude("/healthz"))
            .service(healthz::healthz_app())
            .service(api::internal::internal_app::<CisClient>())
            .service(import::api::import_app::<CisClient>())
            .service(
                web::scope("/groups/api/v1/")
                    .wrap(scope_middleware)
                    .service(api::groups::groups_app::<CisClient>())
                    .service(api::members::members_app::<CisClient>())
                    .service(api::current::current_app::<CisClient>())
                    .service(api::invitations::invitations_app())
                    .service(api::terms::terms_app())
                    .service(api::users::users_app())
                    .service(api::admins::admins_app::<CisClient>())
                    .service(api::requests::requests_app())
                    .service(api::sudo::sudo_app::<CisClient>()),
            )
    })
    .bind("0.0.0.0:8085")?
    .run()
    .await
}
