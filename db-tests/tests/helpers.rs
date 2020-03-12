use dino_park_packs::db::establish_connection;
use dino_park_packs::db::Pool;
use std::env;

pub fn get_pool() -> Pool {
    let pg_url = env::var("DPP_PG_URL").expect("no DPP_PG_URL set");
    println!("DPP_PG_URL: {}", pg_url);
    establish_connection(&pg_url)
}
