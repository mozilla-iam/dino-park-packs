use crate::cis::operations::send_groups_to_cis;
use crate::db::internal;
use crate::db::logs::add_to_comment_body;
use crate::db::operations;
use crate::db::operations::models::*;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::types::*;
use crate::db::Pool;
use crate::error;
use crate::error::PacksError;
use crate::mail::manager::send_email;
use crate::mail::manager::send_emails;
use crate::mail::manager::subscribe_nda;
use crate::mail::manager::unsubscribe_nda;
use crate::mail::templates::Template;
use crate::rules::engine::ADMIN_CAN_ADD_MEMBER;
use crate::rules::engine::ONLY_ADMINS;
use crate::rules::engine::REMOVE_MEMBER;
use crate::rules::engine::RENEW_MEMBER;
use crate::rules::is_nda_group;
use crate::rules::RuleContext;
use crate::user::User;
use chrono::NaiveDateTime;
use chrono::Utc;
use cis_client::AsyncCisClientTrait;
use diesel::dsl::count;
use diesel::prelude::*;
use dino_park_gate::scope::ScopeAndUser;
use dino_park_trust::Trust;
use failure::Error;
use futures::future::try_join_all;
use futures::TryFutureExt;
use log::error;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_RENEWAL_DAYS: i64 = 14;

pub fn membership_and_scoped_host(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<Option<DisplayMembershipAndHost>, Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    let group = internal::group::get_group(&connection, group_name)?;
    match scope_and_user.scope {
        Trust::Staff => {
            internal::member::membership_and_staff_host(&connection, group.id, user.user_uuid)
        }
        Trust::Ndaed => {
            internal::member::membership_and_ndaed_host(&connection, group.id, user.user_uuid)
        }
        Trust::Vouched => {
            internal::member::membership_and_vouched_host(&connection, group.id, user.user_uuid)
        }
        Trust::Authenticated => internal::member::membership_and_authenticated_host(
            &connection,
            group.id,
            user.user_uuid,
        ),
        _ => Ok(None),
    }
}

pub fn scoped_members_and_host(
    pool: &Pool,
    group_name: &str,
    scope_and_user: &ScopeAndUser,
    options: MembersQueryOptions,
) -> Result<PaginatedDisplayMembersAndHost, Error> {
    let connection = pool.get()?;
    let group = internal::group::get_group(&connection, group_name)?;
    let curator = if options.privileged {
        let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
        internal::member::role_for(&connection, &user.user_uuid, group_name)?
            .map(|r| r.typ == RoleType::Admin || r.typ == RoleType::Curator)
            .unwrap_or_default()
    } else {
        false
    };
    match &scope_and_user.scope {
        Trust::Staff if options.privileged && curator => {
            internal::member::privileged_staff_scoped_members_and_host(
                &connection,
                group.id,
                options,
            )
        }
        Trust::Staff => {
            internal::member::staff_scoped_members_and_host(&connection, group.id, options)
        }
        Trust::Ndaed => {
            internal::member::ndaed_scoped_members_and_host(&connection, group.id, options)
        }
        Trust::Vouched => internal::member::vouched_scoped_members(&connection, group.id, options),
        Trust::Authenticated => {
            internal::member::authenticated_scoped_members(&connection, group.id, options)
        }
        Trust::Public => internal::member::public_scoped_members(&connection, group.id, options),
    }
}

pub fn member_count(pool: &Pool, group_name: &str) -> Result<i64, Error> {
    let connection = pool.get()?;
    let count = schema::memberships::table
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .select(count(schema::memberships::user_uuid))
        .first(&connection)?;
    Ok(count)
}

pub fn renewal_count(
    pool: &Pool,
    group_name: &str,
    expires_before: Option<NaiveDateTime>,
) -> Result<i64, Error> {
    let expires_before = expires_before
        .unwrap_or_else(|| (Utc::now() + chrono::Duration::days(DEFAULT_RENEWAL_DAYS)).naive_utc());
    let connection = pool.get()?;
    let count = schema::memberships::table
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .filter(schema::memberships::expiration.le(expires_before))
        .select(count(schema::memberships::user_uuid))
        .first(&connection)?;
    Ok(count)
}

fn db_leave(
    host_uuid: &Uuid,
    connection: &PgConnection,
    group_name: &str,
    user: &User,
    force: bool,
    comment: Option<Value>,
) -> Result<(), Error> {
    if !force && internal::admin::is_last_admin(connection, group_name, &user.user_uuid)? {
        return Err(error::PacksError::LastAdmin.into());
    }
    internal::member::remove_from_group(host_uuid, connection, &user.user_uuid, group_name, comment)
}

pub async fn transfer(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    old_user: &User,
    new_user: &User,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ADMIN_CAN_ADD_MEMBER.run(&RuleContext::minimal_with_member_uuid(
        &pool.clone(),
        scope_and_user,
        group_name,
        &host.user_uuid,
        &new_user.user_uuid,
    ))?;
    internal::member::transfer_membership(&connection, group_name, &host, old_user, new_user)?;
    if group_name == "nda" {
        let old_user_profile =
            internal::user::slim_user_profile_by_uuid(&connection, &old_user.user_uuid)?;
        let new_user_profile =
            internal::user::slim_user_profile_by_uuid(&connection, &old_user.user_uuid)?;
        unsubscribe_nda(old_user_profile.email);
        subscribe_nda(new_user_profile.email);
    }
    drop(connection);
    send_groups_to_cis(pool, Arc::clone(&cis_client), &old_user.user_uuid).await?;
    send_groups_to_cis(pool, cis_client, &new_user.user_uuid).await
}

pub async fn add(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    expiration: Option<i32>,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    ADMIN_CAN_ADD_MEMBER.run(&RuleContext::minimal_with_member_uuid(
        &pool.clone(),
        scope_and_user,
        group_name,
        &host.user_uuid,
        &user.user_uuid,
    ))?;
    let connection = pool.get()?;
    let expiration = if expiration.is_none() {
        internal::group::get_group(&connection, group_name)?.group_expiration
    } else {
        expiration
    };
    internal::member::add_to_group(&connection, group_name, host, user, expiration)?;
    let user_profile = internal::user::slim_user_profile_by_uuid(&connection, &user.user_uuid)?;
    if group_name == "nda" {
        subscribe_nda(&user_profile.email)
    }
    drop(connection);
    send_groups_to_cis(pool, cis_client, &user.user_uuid).await
}

pub async fn remove_members_silent(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    members: &[User],
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    REMOVE_MEMBER.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        group_name,
        &host.user_uuid,
    ))?;
    drop(connection);
    let v = members
        .iter()
        .map(|user| {
            let user_uuid = user.user_uuid;
            log::debug!("removing {} for {}", &group_name, user_uuid);
            operations::members::remove_silent(
                pool,
                scope_and_user,
                group_name,
                &host,
                user,
                Arc::clone(&cis_client),
            )
            .map_ok(move |k| {
                log::debug!("removed {} for {}", &group_name, user_uuid);
                k
            })
            .map_err(move |e| {
                log::warn!("failed to remove {} for {}: {}", &group_name, user_uuid, e);
                e
            })
        })
        .collect::<Vec<_>>();
    log::info!("deleting {} members", v.len());
    try_join_all(v)
        .map_err(|_| PacksError::ErrorDeletingMembers)
        .await?;
    Ok(())
}

pub async fn revoke_memberships_by_trust<'a>(
    pool: &Pool,
    mut remove_groups: RemoveGroups<'a>,
    host: &User,
    trust: TrustType,
    cis_client: Arc<impl AsyncCisClientTrait>,
    comment: Option<Value>,
) -> Result<(), Error> {
    let connection = pool.get()?;

    let comment = add_to_comment_body("reason", "trust revoked", comment);
    for invited in
        internal::invitation::invited_groups_for_user(&connection, &remove_groups.user.user_uuid)?
            .iter()
            .filter(|i| trust < i.trust)
    {
        internal::invitation::delete(
            &connection,
            &invited.name,
            *host,
            remove_groups.user,
            comment.clone(),
        )?;
    }
    let all_groups = internal::group::groups_for_user(&connection, &remove_groups.user.user_uuid)?;
    let mut revoked_groups = all_groups
        .iter()
        .filter(|g| trust < g.trust)
        .map(|g| g.name.as_str())
        .chain(remove_groups.group_names.iter().copied())
        .collect::<Vec<_>>();
    revoked_groups.dedup();
    remove_groups.group_names = &revoked_groups;
    remove_groups.force = true;

    drop(connection);
    _revoke_membership(pool, remove_groups, host, cis_client, comment).await
}

pub async fn revoke_membership<'a>(
    pool: &Pool,
    remove_groups: RemoveGroups<'a>,
    host: &User,
    cis_client: Arc<impl AsyncCisClientTrait>,
    comment: Option<Value>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let is_staff =
        internal::user::user_trust(&connection, &remove_groups.user.user_uuid)? == TrustType::Staff;
    // are we dropping nda membership -> remove according groups and invitations
    if remove_groups
        .group_names
        .iter()
        .any(|group_name| *group_name == "nda")
    {
        let user_profile =
            internal::user::user_profile_by_uuid(&connection, &remove_groups.user.user_uuid)?;
        unsubscribe_nda(user_profile.email);
    }
    if remove_groups
        .group_names
        .iter()
        .any(|group_name| is_nda_group(group_name))
        && !is_staff
    {
        let comment = add_to_comment_body("trust", "nda revoked", comment);
        drop(connection);
        revoke_memberships_by_trust(
            pool,
            remove_groups,
            host,
            TrustType::Authenticated,
            cis_client,
            comment,
        )
        .await
    } else {
        drop(connection);
        _revoke_membership(pool, remove_groups, host, cis_client, comment).await
    }
}
async fn _revoke_membership<'a>(
    pool: &Pool,
    remove_groups: RemoveGroups<'a>,
    host: &User,
    cis_client: Arc<impl AsyncCisClientTrait>,
    comment: Option<Value>,
) -> Result<(), Error> {
    let RemoveGroups {
        user,
        group_names,
        force,
        notify,
    } = remove_groups;
    if group_names.is_empty() {
        return Ok(());
    }
    let exit_on_error = group_names.len() == 1;
    let connection = pool.get()?;
    let user_profile_slim =
        internal::user::slim_user_profile_by_uuid(&connection, &user.user_uuid)?;
    for group_name in group_names {
        if let Err(e) = db_leave(
            &host.user_uuid,
            &connection,
            group_name,
            &user,
            force,
            comment.clone(),
        ) {
            if exit_on_error {
                return Err(e);
            } else {
                error!(
                    "({}) failed to revoke group membership of group {} for {}",
                    e, &group_name, user.user_uuid
                );
            }
        }
        if notify {
            send_email(
                user_profile_slim.email.clone(),
                &Template::DeleteMember(group_name.to_string()),
            );
        }
    }
    drop(connection);
    log::debug!("removing group from profile");
    send_groups_to_cis(pool, cis_client, &user.user_uuid).await?;
    log::debug!("removed group from profile");
    Ok(())
}

pub async fn remove_silent(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    REMOVE_MEMBER.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        group_name,
        &host.user_uuid,
    ))?;
    let remove_groups = RemoveGroups {
        user: *user,
        group_names: &[group_name],
        force: true,
        notify: false,
    };
    revoke_membership(pool, remove_groups, host, cis_client, None).await
}

pub async fn remove(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    REMOVE_MEMBER.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        group_name,
        &host.user_uuid,
    ))?;
    let remove_groups = RemoveGroups {
        user: *user,
        group_names: &[group_name],
        force: true,
        notify: true,
    };
    revoke_membership(pool, remove_groups, host, cis_client, None).await
}

pub async fn leave(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    force: bool,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    let host = User::default();
    let remove_groups = RemoveGroups {
        user,
        group_names: &[group_name],
        force,
        notify: true,
    };
    revoke_membership(pool, remove_groups, &host, cis_client, None).await
}

pub fn renew(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    expiration: Option<i32>,
) -> Result<(), Error> {
    RENEW_MEMBER.run(&RuleContext::minimal_with_member_uuid(
        pool,
        scope_and_user,
        group_name,
        &host.user_uuid,
        &user.user_uuid,
    ))?;
    let connection = pool.get()?;

    internal::member::renew(&host.user_uuid, &connection, group_name, user, expiration)
}

pub fn role_for_current(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<Option<RoleType>, Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;

    internal::member::role_for(&connection, &user.user_uuid, group_name)
        .map(|role| role.map(|role| role.typ))
}

pub fn get_curator_emails(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<Vec<String>, Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        &pool.clone(),
        scope_and_user,
        group_name,
        &user.user_uuid,
    ))?;

    internal::member::get_curator_emails_by_group_name(&connection, group_name)
}

pub fn get_member_emails(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<Vec<String>, Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        &pool.clone(),
        scope_and_user,
        group_name,
        &user.user_uuid,
    ))?;

    internal::member::get_member_emails_by_group_name(&connection, group_name)
}

pub fn get_anonymous_member_emails(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
) -> Result<Vec<String>, Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        &pool.clone(),
        scope_and_user,
        "",
        &user.user_uuid,
    ))?;

    internal::member::get_anonymous_member_emails(&connection)
}

pub fn notify_anonymous_members(pool: &Pool) -> Result<(), Error> {
    let connection = pool.get()?;
    let emails = internal::member::get_anonymous_member_emails(&connection)?;

    send_emails(emails, &Template::AnonymousMember);

    Ok(())
}
