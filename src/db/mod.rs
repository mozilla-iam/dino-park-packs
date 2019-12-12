pub mod error;
mod internal;
pub mod model;
pub mod operations;
#[allow(unused_imports)]
pub mod schema;
pub mod types;
pub mod users;
pub mod views;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection(database_url: &str) -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
