use crate::db::internal;
use crate::db::types::TrustType;
use crate::db::users::DisplayUser;
use crate::db::users::UserForGroup;
use crate::db::users::UserProfile;
use crate::db::Pool;
use crate::rules::engine::SEARCH_USERS;
use crate::rules::is_nda_group;
use crate::rules::RuleContext;
use crate::user::User;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use uuid::Uuid;

pub fn batch_update_user_cache(pool: &Pool, profiles: Vec<Profile>) -> Result<usize, Error> {
    let connection = pool.get()?;
    let l = profiles.len();
    for profile in profiles {
        internal::user::update_user_cache(&connection, &profile)?;
    }
    Ok(l)
}

pub fn search_all_users(
    pool: &Pool,
    scope_and_user: ScopeAndUser,
    trust: Option<TrustType>,
    q: &str,
    limit: i64,
) -> Result<Vec<DisplayUser>, Error> {
    let connection = pool.get()?;

    internal::user::search_users(
        &connection,
        trust.unwrap_or_else(|| TrustType::Authenticated),
        scope_and_user.scope.into(),
        q,
        limit,
    )
}

pub fn search_users(
    pool: &Pool,
    scope_and_user: ScopeAndUser,
    group_name: String,
    trust: Option<TrustType>,
    q: &str,
) -> Result<Vec<UserForGroup>, Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    SEARCH_USERS.run(&RuleContext::minimal(
        pool,
        &scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;

    let trust = if let Some(trust) = trust {
        trust
    } else if is_nda_group(&group_name) {
        TrustType::Authenticated
    } else {
        TrustType::Ndaed
    };

    internal::user::search_users_for_group(
        &connection,
        &group_name,
        trust,
        scope_and_user.scope.into(),
        q,
        5,
    )
}

pub fn search_admins(
    pool: &Pool,
    scope_and_user: ScopeAndUser,
    group_name: String,
    q: &str,
) -> Result<Vec<UserForGroup>, Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    SEARCH_USERS.run(&RuleContext::minimal(
        pool,
        &scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;

    internal::user::search_curators_for_group(
        &connection,
        &group_name,
        scope_and_user.scope.into(),
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
