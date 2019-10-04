#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_derive_enum;

mod types;
mod healthz;
mod user;
mod db;
mod groups;
mod some;

use actix_web::middleware::Logger;
use actix_web::App;
use actix_web::HttpServer;
use failure::Error;
use log::info;

fn main() -> Result<(), Error> {

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    info!("starting dino-park-whoami");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default().exclude("/healthz"))
            .service(healthz::healthz_app())
            .service(some::some_app())
    })
    .bind("0.0.0.0:8085")?
    .run()
    .map_err(Into::into)
}