use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::operations::members::revoke_memberships_by_trust;
use crate::db::types::TrustType;
use crate::db::users::trust_for_profile;
use crate::db::users::DisplayUser;
use crate::db::users::UserForGroup;
use crate::db::users::UserProfile;
use crate::db::Pool;
use crate::error::PacksError;
use crate::rules::engine::SEARCH_USERS;
use crate::rules::RuleContext;
use crate::user::User;
use cis_client::AsyncCisClientTrait;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use std::sync::Arc;
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

    let group = internal::group::get_group(&connection, &group_name)?;

    let trust = if let Some(trust) = trust {
        trust
    } else {
        group.trust
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

pub fn update_user_cache_unchecked(
    pool: &Pool,
    profile: &Profile,
) -> Result<(), Error> {
    let connection = pool.get()?;
    internal::user::update_user_cache(&connection, profile)
}

pub async fn update_user_cache(
    pool: &Pool,
    profile: &Profile,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let new_trust = trust_for_profile(&profile);
    internal::user::update_user_cache(&connection, profile)?;

    let uuid = Uuid::parse_str(&profile.uuid.value.clone().ok_or(PacksError::NoUuid)?)?;
    if let Some(old_profile) = internal::user::user_profile_by_uuid_maybe(&connection, &uuid)? {
        let old_trust = trust_for_profile(&old_profile.profile);
        drop(connection);
        if new_trust < old_trust {
            revoke_memberships_by_trust(
                pool,
                &[],
                &User::default(),
                &User { user_uuid: uuid },
                true,
                new_trust,
                cis_client,
                log_comment_body("trust revoked by CIS update"),
            )
            .await?;
        }
    }
    Ok(())
}

pub fn user_by_id(pool: &Pool, user_id: &str) -> Result<User, Error> {
    let connection = pool.get()?;
    internal::user::user_by_id(&connection, user_id)
}

pub fn user_profile_by_uuid(pool: &Pool, user_uuid: &Uuid) -> Result<UserProfile, Error> {
    let connection = pool.get()?;
    internal::user::user_profile_by_uuid(&connection, user_uuid)
}
