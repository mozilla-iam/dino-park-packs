use crate::cis::operations::send_groups_to_cis;
use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::operations::members::revoke_memberships_by_trust;
use crate::db::operations::models::RemoveGroups;
use crate::db::types::TrustType;
use crate::db::users::trust_for_profile;
use crate::db::users::DisplayUser;
use crate::db::users::UserForGroup;
use crate::db::users::UserProfile;
use crate::db::Pool;
use crate::error::PacksError;
use crate::rules::engine::ONLY_ADMINS;
use crate::rules::engine::SEARCH_USERS;
use crate::rules::RuleContext;
use crate::user::User;
use cis_client::AsyncCisClientTrait;
use cis_profile::schema::KeyValue;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use log::error;
use log::info;
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
        trust.unwrap_or(TrustType::Authenticated),
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

    let trust = trust.unwrap_or(group.trust);

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

pub fn update_user_cache_unchecked(pool: &Pool, profile: &Profile) -> Result<(), Error> {
    let connection = pool.get()?;
    internal::user::update_user_cache(&connection, profile)
}

pub async fn update_user_cache(
    pool: &Pool,
    profile: &Profile,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let user_uuid = Uuid::parse_str(&profile.uuid.value.clone().ok_or(PacksError::NoUuid)?)?;
    if profile.active.value == Some(false) {
        return delete_user(pool, &User { user_uuid });
    }
    let connection = pool.get()?;
    let new_trust = trust_for_profile(profile);
    let old_profile = internal::user::user_profile_by_uuid_maybe(&connection, &user_uuid)?;
    internal::user::update_user_cache(&connection, profile)?;

    if let Some(old_profile) = old_profile {
        let old_trust = trust_for_profile(&old_profile.profile);
        drop(connection);
        let remove_groups = RemoveGroups {
            user: User { user_uuid },
            group_names: &[],
            force: true,
            notify: true,
        };
        if new_trust < old_trust {
            revoke_memberships_by_trust(
                pool,
                remove_groups,
                &User::default(),
                new_trust,
                cis_client,
                log_comment_body("trust revoked by CIS update"),
            )
            .await?;
        }
    } else if let Some(ref groups) = profile.access_information.mozilliansorg.values {
        if !groups.0.is_empty() {
            send_groups_to_cis(pool, cis_client, &user_uuid).await?;
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

pub fn delete_inactive_users(pool: &Pool, scope_and_user: &ScopeAndUser) -> Result<(), Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        "",
        &host.user_uuid,
    ))?;
    let inactive_uuids = internal::user::all_inactive(&connection)?;
    drop(connection);
    info!("deleting {} users", inactive_uuids.len());
    for user_uuid in inactive_uuids {
        delete_user(pool, &User { user_uuid })?;
        info!("delete user {}", user_uuid);
    }
    Ok(())
}

pub fn get_all_member_uuids(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
) -> Result<Vec<Uuid>, Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        "",
        &host.user_uuid,
    ))?;
    internal::user::all_members(&connection)
}

pub fn get_all_staff_uuids(pool: &Pool, scope_and_user: &ScopeAndUser) -> Result<Vec<Uuid>, Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        "",
        &host.user_uuid,
    ))?;
    internal::user::all_staff(&connection)
}

pub use internal::user::update_user_cache as _update_user_cache;

pub async fn consolidate_users_with_cis(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    dry_run: bool,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        "",
        &host.user_uuid,
    ))?;
    let all_with_groups = internal::user::all_with_groups(&connection)?;
    for user_uuid in all_with_groups {
        let user_profile = internal::user::user_profile_by_uuid(&connection, &user_uuid)?;
        if let Some(KeyValue(ref groups)) =
            user_profile.profile.access_information.mozilliansorg.values
        {
            if groups.values().any(Option::is_none) {
                let legacy_groups = groups
                    .iter()
                    .filter_map(|(k, v)| if v.is_none() { Some(k.as_str()) } else { None })
                    .collect::<Vec<_>>()
                    .as_slice()
                    .join(", ");
                if dry_run {
                    info!(
                        "would have removed {} for {} ({})",
                        legacy_groups, user_profile.user_uuid, user_profile.email,
                    );
                } else {
                    match send_groups_to_cis(pool, Arc::clone(&cis_client), &user_uuid).await {
                        Err(e) => error!("failed to consolidate groups for {}: {}", &user_uuid, e),
                        Ok(_) => info!(
                            "removed {} for {} ({})",
                            legacy_groups, user_profile.user_uuid, user_profile.email,
                        ),
                    }
                }
            }
        }
    }
    Ok(())
}
