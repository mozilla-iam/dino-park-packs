use crate::db::db::Pool;
use crate::db::operations::internal;
use cis_profile::schema::Profile;
use failure::Error;

pub fn batch_update_user_cache(pool: &Pool, profiles: Vec<Profile>) -> Result<usize, Error> {
    let l = profiles.len();
    for profile in profiles {
        update_user_cache(pool, &profile)?;
    }
    Ok(l)
}

pub use internal::user::update_user_cache;
pub use internal::user::user_by_id;
pub use internal::user::user_profile_by_uuid;
