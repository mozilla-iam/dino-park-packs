#[macro_use]
extern crate diesel_migrations;

use diesel::connection::Connection;
use diesel::PgConnection;
use std::env;

embed_migrations!();

fn main() {
    let pg_url = env::var("DPP_PG_URL").expect("no DPP_PG_URL set");
    let connection = PgConnection::establish(&pg_url).expect("unable to establish db connection");
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
        .expect("error running migrations");
}
