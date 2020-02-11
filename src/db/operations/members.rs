use crate::cis::operations::add_group_to_profile;
use crate::cis::operations::remove_group_from_profile;
use crate::db::internal;
use crate::db::logs::add_to_comment_body;
use crate::db::operations::models::*;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::types::*;
use crate::db::Pool;
use crate::error;
use crate::rules::engine::ONLY_ADMINS;
use crate::rules::engine::REMOVE_MEMBER;
use crate::rules::engine::RENEW_MEMBER;
use crate::rules::is_nda_group;
use crate::rules::RuleContext;
use crate::user::User;
use chrono::NaiveDateTime;
use chrono::Utc;
use cis_client::CisClient;
use diesel::dsl::count;
use diesel::prelude::*;
use dino_park_gate::scope::ScopeAndUser;
use failure::format_err;
use failure::Error;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_RENEWAL_DAYS: i64 = 14;

pub fn scoped_members_and_host(
    pool: &Pool,
    group_name: &str,
    scope: &str,
    query: Option<String>,
    roles: &[RoleType],
    limit: i64,
    offset: Option<i64>,
) -> Result<PaginatedDisplayMembersAndHost, Error> {
    let connection = pool.get()?;
    match scope {
        "staff" => internal::member::staff_scoped_members_and_host(
            &connection,
            group_name,
            query,
            roles,
            limit,
            offset,
        ),
        "ndaed" => internal::member::ndaed_scoped_members(
            &connection,
            group_name,
            query,
            roles,
            limit,
            offset,
        ),
        "vouched" => internal::member::vouched_scoped_members(
            &connection,
            group_name,
            query,
            roles,
            limit,
            offset,
        ),
        "authenticated" => internal::member::authenticated_scoped_members(
            &connection,
            group_name,
            query,
            roles,
            limit,
            offset,
        ),
        "public" => internal::member::public_scoped_members(
            &connection,
            group_name,
            query,
            roles,
            limit,
            offset,
        ),
        _ => Err(format_err!("invalid scope")),
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
    if force || !internal::admin::is_last_admin(&connection, group_name, &user.user_uuid)? {
        return internal::member::remove_from_group(
            host_uuid,
            &connection,
            &user.user_uuid,
            group_name,
            comment,
        );
    }
    Err(error::PacksError::LastAdmin.into())
}

pub async fn add(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    expiration: Option<i32>,
    cis_client: Arc<CisClient>,
) -> Result<(), Error> {
    ONLY_ADMINS.run(&RuleContext::minimal(
        &pool.clone(),
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    let connection = pool.get()?;
    let expiration = if expiration.is_none() {
        internal::group::get_group(&connection, group_name)?.group_expiration
    } else {
        expiration
    };
    internal::member::add_to_group(&connection, &group_name, &host, &user, expiration)?;
    let user_profile = internal::user::user_profile_by_uuid(&connection, &user.user_uuid)?;
    add_group_to_profile(cis_client, group_name.to_owned(), user_profile.profile).await
}

pub async fn revoke_membership(
    pool: &Pool,
    group_names: &[&str],
    host: &User,
    user: &User,
    force: bool,
    cis_client: Arc<CisClient>,
    comment: Option<Value>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let is_staff = internal::user::user_trust(&connection, &user.user_uuid)? == TrustType::Staff;
    // are we droping nda membership -> remove all groups and invitations
    if group_names
        .iter()
        .any(|group_name| is_nda_group(*group_name))
        && !is_staff
    {
        let all_groups = internal::group::groups_for_user(&connection, &user.user_uuid)?;
        let all_groups = all_groups
            .iter()
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>();
        let comment = add_to_comment_body("reason", "nda revoked", comment);
        let invited = internal::invitation::invited_groups_for_user(&connection, &user.user_uuid)?;
        for group in invited {
            internal::invitation::delete(&connection, &group.name, *host, *user, comment.clone())?;
        }
        _revoke_membership(
            &connection,
            &all_groups,
            host,
            user,
            force,
            cis_client,
            comment,
        )
        .await
    } else {
        _revoke_membership(
            &connection,
            group_names,
            host,
            user,
            force,
            cis_client,
            comment,
        )
        .await
    }
}
async fn _revoke_membership(
    connection: &PgConnection,
    group_names: &[&str],
    host: &User,
    user: &User,
    force: bool,
    cis_client: Arc<CisClient>,
    comment: Option<Value>,
) -> Result<(), Error> {
    let user_profile = internal::user::user_profile_by_uuid(&connection, &user.user_uuid)?;
    remove_group_from_profile(cis_client, group_names, user_profile.profile).await?;
    for group_name in group_names {
        db_leave(
            &host.user_uuid,
            &connection,
            &group_name,
            &user,
            force,
            comment.clone(),
        )?;
    }
    Ok(())
}

pub async fn remove(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    cis_client: Arc<CisClient>,
) -> Result<(), Error> {
    REMOVE_MEMBER.run(&RuleContext::minimal(
        &pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    revoke_membership(pool, &[group_name], host, user, true, cis_client, None).await
}

pub async fn leave(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    force: bool,
    cis_client: Arc<CisClient>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    let host = User::default();
    revoke_membership(pool, &[group_name], &host, &user, force, cis_client, None).await
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
