use crate::db::internal;
use crate::db::types::TrustType;
use crate::db::users::DisplayUser;
use crate::db::Pool;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use std::convert::TryFrom;

pub fn batch_update_user_cache(pool: &Pool, profiles: Vec<Profile>) -> Result<usize, Error> {
    let l = profiles.len();
    for profile in profiles {
        update_user_cache(pool, &profile)?;
    }
    Ok(l)
}

pub fn search_users(
    pool: &Pool,
    scope_and_user: ScopeAndUser,
    trust: Option<TrustType>,
    q: &str,
) -> Result<Vec<DisplayUser>, Error> {
    internal::user::search_users(
        pool,
        trust,
        TrustType::try_from(scope_and_user.scope)?,
        q,
        5,
    )
}

pub use internal::user::update_user_cache;
pub use internal::user::user_by_id;
pub use internal::user::user_profile_by_user_id;
pub use internal::user::user_profile_by_uuid;
