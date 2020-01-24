use crate::db::internal;
use crate::db::types::TrustType;
use crate::db::users::DisplayUser;
use crate::db::users::UserProfile;
use crate::db::Pool;
use crate::user::User;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use std::convert::TryFrom;
use uuid::Uuid;

pub fn batch_update_user_cache(pool: &Pool, profiles: Vec<Profile>) -> Result<usize, Error> {
    let connection = pool.get()?;
    let l = profiles.len();
    for profile in profiles {
        internal::user::update_user_cache(&connection, &profile)?;
    }
    Ok(l)
}

pub fn search_users(
    pool: &Pool,
    scope_and_user: ScopeAndUser,
    trust: Option<TrustType>,
    q: &str,
) -> Result<Vec<DisplayUser>, Error> {
    let connection = pool.get()?;
    internal::user::search_users(
        &connection,
        trust,
        TrustType::try_from(scope_and_user.scope)?,
        q,
        5,
    )
}

pub fn delete_user(pool: &Pool, user: &User) -> Result<(), Error> {
    let connection = pool.get()?;
    internal::user::delete_user(&connection, user)
}

pub fn update_user_cache(pool: &Pool, profile: &Profile) -> Result<(), Error> {
    let connection = pool.get()?;
    internal::user::update_user_cache(&connection, profile)
}

pub fn user_by_id(pool: &Pool, user_id: &str) -> Result<User, Error> {
    let connection = pool.get()?;
    internal::user::user_by_id(&connection, user_id)
}

pub fn user_profile_by_uuid(pool: &Pool, user_uuid: &Uuid) -> Result<UserProfile, Error> {
    let connection = pool.get()?;
    internal::user::user_profile_by_uuid(&connection, user_uuid)
}
