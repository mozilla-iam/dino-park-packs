use diesel_migrations::revert_latest_migration;
use dino_park_packs::db::establish_connection;
use dino_park_packs::db::Pool;
use failure::Error;
use std::env;

embed_migrations!();

pub fn get_pool() -> Pool {
    let pg_url = env::var("DPP_PG_URL").expect("no DPP_PG_URL set");
    establish_connection(&pg_url)
}

pub fn reset() -> Result<(), Error> {
    let connection = get_pool().get()?;
    while revert_latest_migration(&connection).is_ok() {}

    embedded_migrations::run(&connection).expect("error running migrations");
    Ok(())
}
