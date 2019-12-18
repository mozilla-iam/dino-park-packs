use crate::cis::operations::add_group_to_profile;
use crate::cis::operations::remove_group_from_profile;
use crate::db::internal;
use crate::db::operations;
use crate::db::operations::error;
use crate::db::operations::models::*;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::types::*;
use crate::db::Pool;
use crate::rules::engine::ONLY_ADMINS;
use crate::rules::engine::REMOVE_MEMBER;
use crate::rules::engine::RENEW_MEMBER;
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
use futures::future::IntoFuture;
use futures::Future;
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
        "ndaed" => internal::member::ndaed_scoped_members_and_host(
            &connection,
            group_name,
            query,
            roles,
            limit,
            offset,
        ),
        "vouched" => internal::member::vouched_scoped_members_and_host(
            &connection,
            group_name,
            query,
            roles,
            limit,
            offset,
        ),
        "authenticated" => internal::member::authenticated_scoped_members_and_host(
            &connection,
            group_name,
            query,
            roles,
            limit,
            offset,
        ),
        "public" => internal::member::public_scoped_members_and_host(
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
    pool: &Pool,
    group_name: &str,
    user: &User,
    force: bool,
) -> Result<(), Error> {
    let group = internal::group::get_group(&pool, group_name)?;
    if force || !internal::admin::is_last_admin(&pool, group.id, &user.user_uuid)? {
        return internal::member::remove_from_group(host_uuid, &pool, &user.user_uuid, group_name);
    }
    Err(error::OperationError::LastAdmin.into())
}

pub fn add(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    expiration: Option<i32>,
    cis_client: Arc<CisClient>,
) -> impl Future<Item = (), Error = Error> {
    let group_name_f = group_name.to_owned();
    let user_uuid_f = user.user_uuid;
    let pool_f = pool.clone();
    ONLY_ADMINS
        .run(&RuleContext::minimal(
            &pool.clone(),
            scope_and_user,
            &group_name,
            &host.user_uuid,
        ))
        .map_err(Into::into)
        .and_then(move |_| {
            if expiration.is_none() {
                internal::group::get_group(&pool, group_name).map(|g| g.group_expiration)
            } else {
                Ok(expiration)
            }
        })
        .and_then(move |expiration| {
            internal::member::add_to_group(&pool, &group_name, &host, &user, expiration)
        })
        .into_future()
        .and_then(move |_| operations::users::user_profile_by_uuid(&pool_f, &user_uuid_f))
        .and_then(move |user_profile| {
            add_group_to_profile(cis_client, group_name_f, user_profile.profile)
        })
}

pub fn remove(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    cis_client: Arc<CisClient>,
) -> impl Future<Item = (), Error = Error> {
    let group_name_f = group_name.to_owned();
    let group_name_ff = group_name.to_owned();
    let pool_f = pool.clone();
    let user_f = *user;
    let host_uuid = host.user_uuid;
    REMOVE_MEMBER
        .run(&RuleContext::minimal(
            &pool,
            scope_and_user,
            &group_name,
            &host.user_uuid,
        ))
        .map_err(Into::into)
        .and_then(|_| operations::users::user_profile_by_uuid(&pool.clone(), &user.user_uuid))
        .into_future()
        .and_then(move |user_profile| {
            remove_group_from_profile(cis_client, &group_name_f, user_profile.profile)
        })
        .and_then(move |_| {
            db_leave(&host_uuid, &pool_f, &group_name_ff, &user_f, true).into_future()
        })
}

pub fn leave(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    force: bool,
    cis_client: Arc<CisClient>,
) -> impl Future<Item = (), Error = Error> {
    let group_name_f = group_name.to_owned();
    let group_name_ff = group_name.to_owned();
    let pool_f = pool.clone();
    operations::users::user_by_id(&pool.clone(), &scope_and_user.user_id)
        .and_then(|user| {
            operations::users::user_profile_by_uuid(&pool.clone(), &user.user_uuid)
                .map(|user_profile| (user_profile, user))
        })
        .into_future()
        .and_then(move |(user_profile, user)| {
            remove_group_from_profile(cis_client, &group_name_f, user_profile.profile)
                .map(move |_| user)
        })
        .and_then(move |user| {
            db_leave(&user.user_uuid, &pool_f, &group_name_ff, &user, force).into_future()
        })
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
    internal::member::renew(&host.user_uuid, pool, group_name, user, expiration)
}

pub use internal::member::member_role;
pub use internal::member::role_for;
